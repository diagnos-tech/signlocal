import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { buildFiles } from "../../extension/generate-manifest.mjs";
import { readProject } from "../../extension/project-config.mjs";

const extensionDir = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "extension");
const project = readProject();
const manifest = JSON.parse(buildFiles(project)["manifest.json"]);

test("the committed manifest and config are current", () => {
  for (const [name, text] of Object.entries(buildFiles(project))) {
    assert.equal(readFileSync(join(extensionDir, name), "utf8"), text, `${name}: run generate-manifest.mjs`);
  }
});

test("the manifest pins the ID the native host allows", () => {
  assert.equal(manifest.key, project.devKey);
  assert.equal(manifest.browser_specific_settings.gecko.id, project.firefoxId);
});

test("the extension can only reach the native host and localhost pages", () => {
  assert.deepEqual(manifest.permissions, ["nativeMessaging"]);
  assert.deepEqual(manifest.content_scripts[0].matches, ["http://localhost/*", "http://127.0.0.1/*"]);
  assert.equal(manifest.manifest_version, 3);
});

test("every script the manifest names exists", () => {
  const files = [
    manifest.background.service_worker,
    ...manifest.background.scripts,
    ...manifest.content_scripts.flatMap((entry) => entry.js),
  ];
  for (const file of files) {
    assert.doesNotThrow(() => readFileSync(join(extensionDir, file)), file);
  }
});

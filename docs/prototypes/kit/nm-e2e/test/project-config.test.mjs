import assert from "node:assert/strict";
import { test } from "node:test";

import { parseProjectToml, readProject } from "../../extension/project-config.mjs";

test("parses sections, string values, comments and escapes", () => {
  const toml = parseProjectToml(`
# a comment
[ids]
native_host = "dev.example.host"   # trailing comment
quoted = "say \\"hi\\""

[extension]
empty = ""
number = 3
`);
  assert.equal(toml.ids.native_host, "dev.example.host");
  assert.equal(toml.ids.quoted, 'say "hi"');
  assert.equal(toml.extension.empty, "");
  assert.equal(toml.extension.number, undefined, "non-string values are ignored");
});

test("ignores pairs that appear before any section", () => {
  assert.deepEqual(parseProjectToml('stray = "x"\n[a]\nk = "v"\n'), { a: { k: "v" } });
});

test("the repository's project.toml provides every identifier", () => {
  const project = readProject();
  assert.match(project.devId, /^[a-p]{32}$/);
  assert.match(project.nativeHost, /^[a-z0-9_.]+$/, "Chrome only accepts [a-z0-9_.] in host names");
  assert.ok(project.devKey.length > 100);
  assert.ok(project.firefoxId.includes("@") || project.firefoxId.startsWith("{"));
});

// The code samples of the developers page → src/generated/snippets.json.
//
// Each sample is a `// #region site:<name>` … `// #endregion site:<name>` block of a file that
// is type-checked or tested elsewhere (examples/), so the page can never show code that does
// not compile. A missing region fails the build with the file name.

import { readFileSync } from "node:fs";
import type { SnippetsData } from "../../src/lib/data-types.ts";

/** Region name → the file that holds it, relative to the repository root. */
export const SNIPPET_SOURCES: Readonly<Record<string, string>> = {
  quickstart: "examples/web/snippets/quickstart.ts",
  status: "examples/web/snippets/status.ts",
  errors: "examples/web/snippets/errors.ts",
  testing: "examples/web/snippets/testing.ts",
  "testing-dev": "examples/web/snippets/testing-dev.ts",
  react: "examples/web/react/useSignlocal.ts",
  "react-usage": "examples/web/snippets/react-usage.tsx",
  pades: "examples/web/pades/sign-pdf.ts",
  node: "examples/desktop/node-cli/sign-file.mjs",
  rust: "examples/desktop/rust-cli/src/main.rs",
};

/** The lines between the region markers, without the common indentation. */
export function extractRegion(source: string, name: string, file: string): string {
  const lines = source.split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === `// #region site:${name}`);
  const end = lines.findIndex((line) => line.trim() === `// #endregion site:${name}`);
  if (start < 0 || end < start) throw new Error(`${file}: region "site:${name}" not found`);
  const body = lines.slice(start + 1, end);
  const indent = Math.min(
    ...body
      .filter((line) => line.trim() !== "")
      .map((line) => line.length - line.trimStart().length),
  );
  return body
    .map((line) => line.slice(indent))
    .join("\n")
    .trimEnd();
}

export function readSnippets(repo: string): SnippetsData {
  const snippets: Record<string, string> = {};
  for (const [name, file] of Object.entries(SNIPPET_SOURCES)) {
    snippets[name] = extractRegion(readFileSync(`${repo}/${file}`, "utf8"), name, file);
  }
  return snippets;
}

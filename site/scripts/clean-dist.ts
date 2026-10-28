// After the build: keeps the folder notes out of the published site and names the custom
// domain (GitHub ignores CNAME when Pages is deployed from Actions, but a copy of the site
// published anywhere else keeps its domain).

import { readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { readProject } from "./data/project.ts";

const site = fileURLToPath(new URL("..", import.meta.url));
const dist = join(site, "dist");

function removeNotes(folder: string): void {
  for (const entry of readdirSync(folder, { withFileTypes: true })) {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) removeNotes(path);
    else if (entry.name === "SUMMARY.md" || entry.name === "README.md") rmSync(path);
  }
}

removeNotes(dist);
writeFileSync(join(dist, "CNAME"), `${new URL(readProject(join(site, "..")).homepage).host}\n`);

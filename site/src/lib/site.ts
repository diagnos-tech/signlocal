// Typed access to the data the generators wrote (scripts/data/ → src/generated/).
import { execFileSync } from "node:child_process";

import project from "../generated/project.json";
import release from "../generated/release.json";
import snippets from "../generated/snippets.json";
import support from "../generated/support.json";
import type { ProjectData, ReleaseData, SnippetsData, SupportData } from "./data-types.ts";

export const PROJECT = project as ProjectData;
export const RELEASE = release as unknown as ReleaseData;
export const SUPPORT = support as SupportData;
export const SNIPPETS = snippets as SnippetsData;

/** The commit the pages were built from, for the post-deploy check (GITHUB_SHA in CI, else git). */
function buildId(): string {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA;
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  } catch {
    return "unknown";
  }
}

export const BUILD_ID = buildId();

/**
 * WXT build configuration: one source, four targets (chrome, edge, firefox,
 * safari). Identifiers come from project.toml through src/generated/project.ts;
 * the manifest fields live in build/manifest.ts (tested).
 */

import { defineConfig } from "wxt";
import { channelFromEnv, manifestFor } from "./build/manifest";

const channel = channelFromEnv(process.env.WEBSIGN_CHANNEL);

export default defineConfig({
  srcDir: "src",
  imports: false,
  manifestVersion: 3,
  manifest: ({ browser, mode }) => manifestFor({ browser, mode, channel }),
  hooks: {
    // Folder maps (SUMMARY.md) document the repository, not the shipped extension.
    "build:publicAssets": (_wxt, files) => {
      const shipped = files.filter((file) => !file.relativeDest.endsWith("SUMMARY.md"));
      files.splice(0, files.length, ...shipped);
    },
  },
});

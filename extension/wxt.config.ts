/**
 * WXT build configuration: one source, four targets (chrome, edge, firefox,
 * safari). Identifiers come from project.toml through src/generated/project.ts.
 */

import { defineConfig } from "wxt";
import { DEV_KEY, FIREFOX_ID } from "./src/generated/project";

export default defineConfig({
  srcDir: "src",
  imports: false,
  manifestVersion: 3,
  manifest: ({ browser, mode }) => ({
    name: "__MSG_extension_name__",
    description: "__MSG_extension_description__",
    default_locale: "en",
    permissions: ["nativeMessaging"],
    minimum_chrome_version: "121",
    // Pins the unpacked development build's ID so the native host can allow it.
    ...(browser !== "firefox" && browser !== "safari" && mode === "development"
      ? { key: DEV_KEY }
      : {}),
    ...(browser === "firefox"
      ? {
          browser_specific_settings: {
            gecko: {
              id: FIREFOX_ID,
              strict_min_version: "121.0",
              data_collection_permissions: { required: ["none"] },
            },
          },
        }
      : {}),
  }),
});

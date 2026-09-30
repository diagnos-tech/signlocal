import { test } from "@playwright/test";

// Scenario list (docs/architecture/testing.md §E2E scenarios); the e2e track
// implements each against the fixture page.
test.describe("sign through the extension", () => {
  test.fixme("signs a SHA-256 digest with each software key and verifies it", async () => {});
  test.fixme("certificates() opens choose mode for a new site", async () => {});
  test.fixme("remembered site gets certificates() without a window", async () => {});
  test.fixme("cancel returns UserCancelled; closing the tab cancels in the app", async () => {});
  test.fixme("reports ExtensionMissing and AppMissing", async () => {});
});

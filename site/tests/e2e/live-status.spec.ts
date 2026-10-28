import { USER_AGENTS, wizardUrl } from "./helpers.ts";
import { expect, test } from "./test.ts";

// Needs a build made with PUBLIC_SIGNLOCAL_TEST_HOOKS=1: the fake extension and app are not
// in the published site.
test.use({ userAgent: USER_AGENTS.windowsChrome });

const link = { os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" };

test("with everything installed, the cards show it and the ready banner appears", async ({
  page,
}) => {
  await page.goto(wizardUrl({ ...link, fake: "ready" }));
  await expect(page.getByText("signlocal is ready in Chrome")).toBeVisible();
  await expect(page.getByRole("link", { name: "Test your setup" }).first()).toBeVisible();
  await expect(page.getByText(/Installed · /).first()).toBeVisible();
  await expect(page.getByText("Connected", { exact: true })).toBeVisible();
});

test("without the extension, the card says it was not detected", async ({ page }) => {
  await page.goto(wizardUrl({ ...link, fake: "extension-missing" }));
  await expect(page.getByText("Not detected yet").filter({ visible: true })).toBeVisible();
  await expect(page.getByText("signlocal is ready in Chrome")).toBeHidden();
});

test("an app older than the extension needs shows the versions", async ({ page }) => {
  await page.goto(wizardUrl({ ...link, fake: "app-outdated" }));
  await expect(page.getByText(/Update needed: .* installed, 0\.1\.0 required/)).toBeVisible();
});

test("with the extension but no app, no app chip is shown", async ({ page }) => {
  await page.goto(wizardUrl({ ...link, fake: "app-missing" }));
  await expect(page.getByText(/Installed · /).first()).toBeVisible();
  await expect(page.getByText("signlocal is ready in Chrome")).toBeHidden();
});

test("the test page and the activate page follow the same status", async ({ page }) => {
  await page.goto("/test/?sdk-fake=ready");
  await expect(page.getByRole("heading", { name: "Everything is installed" })).toBeVisible();
});

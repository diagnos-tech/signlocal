import { USER_AGENTS, wizardUrl } from "./helpers.ts";
import { expect, test } from "./test.ts";

// Runs against a build made with SIGNLOCAL_SITE_RELEASE=none (CI does a second build for it)
// and skips itself against one that has a release.
test.use({ userAgent: USER_AGENTS.windowsChrome });

test("with no published release the steps say so and offer no broken download", async ({
  page,
}) => {
  await page.goto(
    wizardUrl({ os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" }),
  );
  test.skip(
    (await page.locator("[data-wizard]").getAttribute("data-has-release")) === "yes",
    "this build has a release",
  );
  await expect(
    page.getByRole("heading", { name: "The first release is almost ready" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Download" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Get your token ready" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Install the signlocal app" })).toBeHidden();
});

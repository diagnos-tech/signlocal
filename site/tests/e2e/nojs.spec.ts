import { expect, test } from "@playwright/test";

test.use({ javaScriptEnabled: false });

test("the home page reads without JavaScript", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Sign on any website");
  await expect(page.getByRole("link", { name: "Windows" })).toHaveAttribute(
    "href",
    /download\.html\?os=windows$/,
  );
  await expect(page.getByRole("heading", { name: "How it works" })).toBeVisible();
});

test("the install page is a complete written guide without JavaScript", async ({ page }) => {
  await page.goto("/download.html");
  await expect(page.getByText("Below are the steps for every system and browser.")).toBeVisible();
  await expect(page.getByRole("heading", { name: /Which system/ })).toBeHidden();
  for (const name of ["Windows", "macOS", "Chrome, Edge, Opera, Brave", "Firefox"]) {
    await expect(page.getByRole("heading", { name, level: 4 }).first()).toBeVisible();
  }
  await expect(
    page
      .getByRole("heading", { name: /Linux · Ubuntu, Debian, Mint or similar · Intel or AMD/ })
      .first(),
  ).toBeVisible();
  const downloads = page.getByRole("link", { name: "Download" });
  expect(await downloads.count()).toBeGreaterThanOrEqual(10);
  await expect(page.getByRole("button", { name: "Copy" })).toHaveCount(0);
});

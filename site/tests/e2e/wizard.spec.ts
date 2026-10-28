import { recipe } from "../../src/wizard/recipe.ts";
import { RESULTS, USER_AGENTS, visibleCards, wizardUrl } from "./helpers.ts";
import { expect, test } from "./test.ts";

test.describe("on Windows with Chrome", () => {
  test.use({ userAgent: USER_AGENTS.windowsChrome });

  test("a keyboard alone gets to the steps, and the address says where we are", async ({
    page,
  }) => {
    await page.goto(wizardUrl());
    const heading = (name: RegExp) => page.getByRole("heading", { name, level: 2 });
    await expect(heading(/Which system/)).toBeVisible();
    await page.getByRole("radio", { name: /Windows/ }).focus();
    await page.keyboard.press("Enter");
    await expect(heading(/Which browser/)).toBeFocused();
    await expect(page).toHaveURL(/step=browser/);
    await page.keyboard.press("Tab");
    await page.keyboard.press("Enter");
    await expect(heading(/Where is your certificate/)).toBeFocused();
    await page.getByRole("radio", { name: /USB token/ }).focus();
    await page.keyboard.press("Space");
    await page.keyboard.press("Enter");
    await expect(heading(/Install signlocal on Windows for Chrome/)).toBeFocused();
    await expect(page).toHaveURL(/os=windows&arch=x64&browser=chrome&cert=token&step=result/);
  });

  test("detection preselects the system, processor and browser but never the certificate", async ({
    page,
  }) => {
    await page.goto(wizardUrl());
    await expect(page.getByRole("radio", { name: /Windows/ })).toBeChecked();
    await expect(page.getByRole("radio", { name: /Intel or AMD/ })).toBeChecked();
    await page.getByRole("button", { name: "Next" }).click();
    await expect(page.getByRole("radio", { name: "Chrome", exact: true })).toBeChecked();
    await page.getByRole("button", { name: "Next" }).click();
    for (const radio of await page
      .getByRole("radio", { name: /USB token|Smart card|Installed on this computer|not sure/ })
      .all()) {
      await expect(radio).not.toBeChecked();
    }
  });

  test("the browser's Back and Forward buttons walk the steps", async ({ page }) => {
    await page.goto(wizardUrl());
    await page.getByRole("button", { name: "Next" }).click();
    await page.getByRole("button", { name: "Next" }).click();
    await expect(page.getByRole("heading", { name: /Where is your certificate/ })).toBeVisible();
    await page.goBack();
    await expect(page.getByRole("heading", { name: /Which browser/ })).toBeVisible();
    await page.goBack();
    await expect(page.getByRole("heading", { name: /Which system/ })).toBeVisible();
    await page.goForward();
    await expect(page.getByRole("heading", { name: /Which browser/ })).toBeVisible();
  });

  test("Next on an empty question explains and focuses the first option", async ({ page }) => {
    await page.goto(
      wizardUrl({ os: "windows", arch: "x64", browser: "chrome", step: "certificate" }),
    );
    await page.getByRole("button", { name: "Show my steps" }).click();
    await expect(
      page.getByRole("alert").filter({ hasText: "Choose where your certificate is" }),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: /USB token/ })).toBeFocused();
  });

  test("a full link opens the steps, with the difference from this computer pointed out", async ({
    page,
  }) => {
    await page.goto(wizardUrl({ os: "macos", browser: "safari", cert: "file", step: "result" }));
    await expect(
      page.getByRole("heading", { name: "Install signlocal on macOS for Safari" }),
    ).toBeVisible();
    await expect(
      page.getByText("These steps are for macOS, but this computer looks like Windows."),
    ).toBeVisible();
    await page.getByRole("button", { name: "Show the steps for Windows" }).click();
    await expect(page).toHaveURL(/os=windows/);
  });

  test("changing the system away from the Mac removes Safari with a notice", async ({ page }) => {
    await page.goto(wizardUrl({ os: "macos", browser: "safari", step: "system" }));
    await page.getByRole("radio", { name: /Windows/ }).check();
    await page.getByRole("button", { name: "Next" }).click();
    await expect(
      page.getByText("Safari isn't available for Windows. Choose another browser."),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: /Safari/ })).toBeHidden();
  });

  test("the legacy hash of the popup opens the Browser step", async ({ page }) => {
    await page.goto("/download.html#macos");
    await expect(page.getByRole("heading", { name: /Which browser/ })).toBeVisible();
    await expect(page.getByRole("radio", { name: /Safari/ })).toBeVisible();
    await expect(page).toHaveURL(/\?os=macos&step=browser$/);
  });

  test("Safari is offered on macOS only", async ({ page }) => {
    await page.goto(wizardUrl({ os: "windows", arch: "x64", step: "browser" }));
    await expect(page.getByRole("radio", { name: /Safari/ })).toBeHidden();
    await page.goto(wizardUrl({ os: "macos", step: "browser" }));
    await expect(page.getByRole("radio", { name: /Safari/ })).toBeVisible();
  });

  test("a certificate file on Linux says it is not supported yet", async ({ page }) => {
    await page.goto(
      wizardUrl({
        os: "linux",
        arch: "x64",
        distro: "deb",
        browser: "chrome",
        cert: "file",
        step: "result",
      }),
    );
    await expect(
      page.getByText("Certificate files don't work on Linux yet").filter({ visible: true }),
    ).toBeVisible();
  });

  test("Copy link copies this page's address with the choices, opening on the steps", async ({
    page,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.goto(
      wizardUrl({ os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" }),
    );
    await page.getByRole("button", { name: "Copy link to these steps" }).click();
    const copied = await page.evaluate(() => navigator.clipboard.readText());
    expect(new URL(copied).pathname).toBe("/download.html");
    expect(new URL(copied).search).toBe(
      "?os=windows&arch=x64&browser=chrome&cert=token&step=result",
    );
  });

  test("the device list names the driver and where to get it for this system", async ({ page }) => {
    await page.goto(
      wizardUrl({ os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" }),
    );
    await page.locator("#device-main-token").selectOption("safenet-etoken-5110");
    await expect(
      page
        .getByText("SafeNet eToken 5110 uses SafeNet Authentication Client.")
        .filter({ visible: true }),
    ).toBeVisible();
    await expect(
      page
        .getByRole("link", { name: /Download SafeNet Authentication Client/ })
        .filter({ visible: true }),
    ).toBeVisible();
    await expect(page).toHaveURL(/device=safenet-etoken-5110/);
  });

  test("the visible cards are exactly the recipe, for six combinations", async ({ page }) => {
    const combos = [
      ...RESULTS.map((result) => result.link),
      { os: "windows", arch: "arm64", browser: "opera", cert: "card", step: "result" },
      { os: "macos", browser: "firefox", cert: "unsure", step: "result" },
    ];
    for (const link of combos) {
      await page.goto(wizardUrl(link));
      await page.getByRole("heading", { level: 2 }).first().waitFor();
      expect(await visibleCards(page)).toEqual(recipe({ browser: link.browser as never }, true));
    }
  });

  test("the extension card follows the browser: developer install for Chrome, a warning for Firefox, settings for Safari", async ({
    page,
  }) => {
    await page.goto(
      wizardUrl({ os: "windows", arch: "x64", browser: "chrome", cert: "token", step: "result" }),
    );
    await expect(page.getByText("The extension isn't in the browser stores yet")).toBeVisible();
    await expect(page.locator("#address-chrome")).toBeVisible();
    await page.goto(
      wizardUrl({ os: "windows", arch: "x64", browser: "firefox", cert: "token", step: "result" }),
    );
    await expect(page.getByText("The extension isn't on Firefox Add-ons yet")).toBeVisible();
    await page.goto(wizardUrl({ os: "macos", browser: "safari", cert: "token", step: "result" }));
    await expect(
      page.getByRole("heading", { name: "Turn on the extension in Safari" }),
    ).toBeVisible();
  });

  test("Linux packages open on the manual download, other systems on the automatic install", async ({
    page,
  }) => {
    await page.goto(
      wizardUrl({
        os: "linux",
        arch: "x64",
        distro: "deb",
        browser: "chrome",
        cert: "token",
        step: "result",
      }),
    );
    await expect(page.getByRole("tab", { name: "Manual download" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await page.goto(
      wizardUrl({
        os: "linux",
        arch: "x64",
        distro: "other",
        browser: "chrome",
        cert: "token",
        step: "result",
      }),
    );
    await expect(page.getByRole("tab", { name: /Automatic/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });
});

test.describe("with no system detected", () => {
  test.use({ userAgent: USER_AGENTS.unknown });

  test("nothing is preselected and Next asks for a system", async ({ page }) => {
    // The browser's own client hints would still name the host's system.
    await page.addInitScript(() =>
      Object.defineProperty(navigator, "userAgentData", { get: () => undefined }),
    );
    await page.goto(wizardUrl());
    await expect(page.getByText("Choose the system of the computer where you sign.")).toBeVisible();
    await page.getByRole("button", { name: "Next" }).click();
    await expect(
      page.getByRole("alert").filter({ hasText: "Choose your system to continue." }),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: /Windows/ })).toBeFocused();
  });
});

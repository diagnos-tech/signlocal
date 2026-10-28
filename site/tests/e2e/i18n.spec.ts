import { USER_AGENTS, wizardUrl } from "./helpers.ts";
import { expect, test } from "./test.ts";

test.describe("a visitor whose browser prefers Brazilian Portuguese", () => {
  test.use({ locale: "pt-BR", userAgent: USER_AGENTS.macSafari });

  test("the popup's download.html#macos opens the Portuguese wizard at the browser step", async ({
    page,
  }) => {
    await page.goto("/download.html#macos");
    await expect(page).toHaveURL(/\/pt-br\/download\.html\?os=macos&step=browser$/);
    await expect(
      page.getByRole("heading", { name: "Em qual navegador você assina?" }),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: /Safari/ })).toBeVisible();
  });

  test("?lang=en keeps English and is remembered", async ({ page }) => {
    await page.goto("/?lang=en");
    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Sign on any website");
    await page.goto("/download.html");
    await expect(page).toHaveURL(/\/download\.html/);
    expect(await page.evaluate(() => localStorage.getItem("lang"))).toBe("en");
  });

  test("a shared Portuguese link stays Portuguese for a visitor who prefers English", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "en-US" });
    const page = await context.newPage();
    await page.goto("/pt-br/");
    await expect(page).toHaveURL(/\/pt-br\/$/);
    await context.close();
  });

  test("the language menu keeps the wizard's choices", async ({ page }) => {
    await page.goto(wizardUrl({ os: "macos", browser: "chrome", step: "certificate" }, "/pt-br"));
    await page.locator("summary:has-text('Idioma')").first().click();
    const link = page.getByRole("link", { name: "Deutsch" });
    await expect(link).toHaveAttribute(
      "href",
      /\/de\/download\.html\?os=macos&browser=chrome&step=certificate&lang=de$/,
    );
  });

  test("English-only pages never redirect", async ({ page }) => {
    await page.goto("/developers.html");
    await expect(page).toHaveURL(/\/developers\.html$/);
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Add signing");
  });
});

// Page ids → URLs. Translated pages exist once per language; the developer pages and the
// notices exist in English only, at the root.

import { folderOf, type LocaleCode } from "../i18n/codes.ts";
import { LOCALES } from "../i18n/locales.ts";
import type { LinkResolver } from "../i18n/rich.ts";
import type { ProjectData } from "./data-types.ts";

export type PageId =
  | "home"
  | "install"
  | "test"
  | "activate"
  | "privacy"
  | "compatibility"
  | "help"
  | "developers"
  | "notices";

const FILES: Readonly<Record<PageId, string>> = {
  home: "",
  install: "download.html",
  test: "test/",
  activate: "activate/",
  privacy: "privacy.html",
  compatibility: "compatibility.html",
  help: "help.html",
  developers: "developers.html",
  notices: "notices.html",
};

const ENGLISH_ONLY: ReadonlySet<PageId> = new Set(["developers", "notices"]);

export const isTranslated = (id: PageId): boolean => !ENGLISH_ONLY.has(id);

/** The path of a page in a language, with its fragment: "/pt-br/download.html#windows". */
export function pagePath(id: PageId, locale: LocaleCode, fragment?: string): string {
  const folder = isTranslated(id) ? folderOf(locale) : "";
  const path = `${folder}/${FILES[id]}`;
  return fragment ? `${path}#${fragment}` : path;
}

export function pageUrl(id: PageId, locale: LocaleCode, homepage: string): string {
  return new URL(pagePath(id, locale), homepage).href;
}

/** `hreflang` alternates of a translated page: every language plus x-default (English). */
export function alternates(id: PageId, homepage: string): { hreflang: string; href: string }[] {
  const all = LOCALES.map(({ code }) => ({ hreflang: code, href: pageUrl(id, code, homepage) }));
  return [...all, { hreflang: "x-default", href: pageUrl(id, "en", homepage) }];
}

/** Where the page ids of the copy's [label](id) links point; unknown ids fail the build. */
export function linkResolver(locale: LocaleCode, project: ProjectData): LinkResolver {
  const external: Readonly<Record<string, string>> = {
    releases: `${project.repository}/releases`,
    repository: project.repository,
    issues: `${project.repository}/issues`,
    security: `${project.repository}/security/policy`,
  };
  return (id, fragment) => {
    const url = external[id];
    if (url) return url;
    if (id in FILES) return pagePath(id as PageId, locale, fragment);
    throw new Error(`Unknown page id "${id}" in a message link`);
  };
}

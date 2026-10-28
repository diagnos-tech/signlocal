// What the post-build checks read out of a built page, with Bun's streaming HTML parser.

import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

export interface Page {
  /** Path from dist/ with forward slashes: "pt-br/download.html". */
  readonly path: string;
  readonly html: string;
}

export interface Attribute {
  readonly tag: string;
  readonly name: string;
  readonly value: string;
  readonly rel?: string | undefined;
}

export interface Facts {
  /** Every href, src and srcset-less URL attribute, with its tag. */
  readonly urls: readonly Attribute[];
  readonly ids: ReadonlySet<string>;
  readonly h1Count: number;
  readonly title: string;
  readonly meta: Readonly<Record<string, string>>;
  readonly canonical: string | undefined;
  readonly alternates: readonly { hreflang: string; href: string }[];
  readonly lang: string;
  readonly inlineStyles: number;
  readonly styleAttributes: number;
  readonly eventAttributes: number;
  readonly inlineScripts: readonly string[];
  readonly csp: string | undefined;
}

/** The HTML files of a folder, recursively, skipping the folders named in `skip` (e.g. api). */
export function pagesIn(dist: string, skip: readonly string[] = ["api"]): Page[] {
  const pages: Page[] = [];
  const walk = (folder: string) => {
    for (const entry of readdirSync(folder, { withFileTypes: true })) {
      const path = join(folder, entry.name);
      const rel = relative(dist, path).split("\\").join("/");
      if (entry.isDirectory()) {
        if (!skip.includes(rel)) walk(path);
      } else if (entry.name.endsWith(".html")) {
        pages.push({ path: rel, html: readFileSync(path, "utf8") });
      }
    }
  };
  walk(dist);
  return pages;
}

export async function factsOf(html: string): Promise<Facts> {
  const urls: Attribute[] = [];
  const ids = new Set<string>();
  const meta: Record<string, string> = {};
  const alternates: { hreflang: string; href: string }[] = [];
  const inlineScripts: string[] = [];
  let h1Count = 0;
  let title = "";
  let canonical: string | undefined;
  let lang = "";
  let inlineStyles = 0;
  let styleAttributes = 0;
  let eventAttributes = 0;
  let csp: string | undefined;
  let scriptType: string | null = null;
  const rewriter = new HTMLRewriter()
    .on("html", {
      element: (element) => {
        lang = element.getAttribute("lang") ?? "";
      },
    })
    .on("h1", {
      element: () => {
        h1Count += 1;
      },
    })
    .on("title", {
      text: (chunk) => {
        title += chunk.text;
      },
    })
    .on("style", {
      element: () => {
        inlineStyles += 1;
      },
    })
    .on("meta", {
      element: (element) => {
        const name = element.getAttribute("name") ?? element.getAttribute("property");
        if (name) meta[name] = element.getAttribute("content") ?? "";
        if (element.getAttribute("http-equiv")?.toLowerCase() === "content-security-policy") {
          csp = element.getAttribute("content") ?? "";
        }
      },
    })
    .on("link", {
      element: (element) => {
        const rel = element.getAttribute("rel") ?? "";
        const href = element.getAttribute("href") ?? "";
        if (rel === "canonical") canonical = href;
        if (rel === "alternate" && element.getAttribute("hreflang")) {
          alternates.push({ hreflang: element.getAttribute("hreflang") ?? "", href });
        }
        if (href) urls.push({ tag: "link", name: "href", value: href, rel });
      },
    })
    .on("a, img, script, source", {
      element: (element) => {
        for (const name of ["href", "src"]) {
          const value = element.getAttribute(name);
          if (value !== null) urls.push({ tag: element.tagName, name, value });
        }
      },
    })
    .on("script", {
      element: (element) => {
        scriptType = element.getAttribute("type");
        if (element.getAttribute("src") === null)
          inlineScripts.push(scriptType ?? "text/javascript");
      },
    })
    .on("*", {
      element: (element) => {
        const id = element.getAttribute("id");
        if (id) ids.add(id);
        if (element.tagName === "a" && element.getAttribute("name"))
          ids.add(element.getAttribute("name") ?? "");
        if (element.hasAttribute("style")) styleAttributes += 1;
        for (const [name] of element.attributes) if (/^on[a-z]+$/.test(name)) eventAttributes += 1;
      },
    });
  await rewriter.transform(new Response(html)).text();
  return {
    urls,
    ids,
    h1Count,
    title: title.trim(),
    meta,
    canonical,
    alternates,
    lang,
    inlineStyles,
    styleAttributes,
    eventAttributes,
    inlineScripts,
    csp,
  };
}

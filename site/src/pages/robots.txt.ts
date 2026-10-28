// Everything may be crawled; the sitemap lists the pages worth indexing.

import type { APIRoute } from "astro";
import { PROJECT } from "../lib/site.ts";

export const GET: APIRoute = () =>
  new Response(
    `User-agent: *\nAllow: /\n\nSitemap: ${new URL("sitemap-index.xml", PROJECT.homepage).href}\n`,
    {
      headers: { "Content-Type": "text/plain; charset=utf-8" },
    },
  );

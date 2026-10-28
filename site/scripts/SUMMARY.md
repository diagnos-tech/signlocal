# site/scripts

Everything that runs around Astro: data generators, post-build checks, the preview server, the image renderer.

- `check/` — post-build gates on dist/: links, Content-Security-Policy rules, SEO basics, size budgets
- `clean-dist.ts` — removes folder notes from dist/ and writes CNAME
- `data/` — generators that write src/generated/ before the build (release, project, support, snippets, message keys, boot script)
- `images/` — renders the social image and the home-screen icon from their HTML sources
- `preload-modules.ts` — lists each page's imported modules in its head so they load in one round trip
- `serve.ts` — serves dist/ with GitHub Pages' rules (x → x.html, x/ → x/index.html, 404.html)

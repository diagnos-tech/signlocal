# site/src

The site's source: routes, views, components, texts and logic.

- `brand/` — the mark and the ink underline
- `brands/` — browser logos (third-party marks, unmodified) and their licence
- `client/` — scripts that run in the browser
- `components/` — Astro components, one folder per area
- `content/` — page content that is data: the questions and the error descriptions
- `fonts/` — Inter and JetBrains Mono, self-hosted
- `i18n/` — the seven languages: codes, messages, rich text, language choice
- `icons/` — Phosphor icons, inlined as SVG
- `layouts/` — the frame shared by the end-user pages
- `lib/` — build-time helpers: routes, data access, formatting, icons
- `pages/` — the routes (`[lang]/` repeats the translated pages for six languages; every route file only picks a view). No folder notes inside: Astro would publish a Markdown file there as a page
- `platform/` — what the wizard knows about systems, browsers, files and commands (pure, shared by build and browser)
- `styles/` — global CSS: fonts, base, layout, utilities
- `views/` — one component per page, rendered for every language
- `wizard/` — the install wizard's state machine and browser code

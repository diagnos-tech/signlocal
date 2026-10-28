# site/src/client

Scripts that run in the browser. None runs at import time: pages call each module's init function.

- `activate.ts` — the Finish setting up page: follows the connection after the button is pressed
- `boot.ts` — the blocking first script (public/boot.js): the js class and the language redirect, under 1 KB
- `copy.ts` — copy buttons and their status messages
- `device-filter.ts` — the filter of the compatibility page's device table
- `device-notice.ts` — the phone notice: share, copy link, e-mail link
- `home.ts` — the home page's start card: detected system, phone notice
- `language-menu.ts` — closes the language menus; keeps the page's parameters on their links
- `strings.ts` — reads the texts a page rendered for a module
- `tabs.ts` — ARIA tabs with automatic activation
- `test-page/` — the test page: signs a sample and checks the signature in the browser

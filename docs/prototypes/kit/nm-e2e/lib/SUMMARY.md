# nm-e2e/lib

Modules of the end-to-end harness, one concern per file.

- `browser.mjs` — launches Chromium with the test extension loaded
- `cli.mjs` — parses the command line of `run.mjs` and prints usage
- `host-log.mjs` — reads the host's diagnostic log, the evidence of what happened between browser and host
- `page-server.mjs` — serves `page.html` on a random localhost port
- `verdict.mjs` — judges what the page saw and what the host logged

/**
 * Serves the pages under test from the repository on http://localhost: the
 * fixture page (`/`, `e2e/fixtures/`) and the website (`/site/`, whose
 * `test/index.html` is the real "test your setup" page using the SDK
 * bundle). The extension injects itself into localhost pages, and a named
 * host (not 127.0.0.1) can be remembered.
 *
 * Every server gets its own port, so every server is a new site to the app.
 */

import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, normalize, sep } from "node:path";

import { REPO } from "./environment.ts";

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".json": "application/json",
};

const ROOTS: ReadonlyArray<readonly [string, string]> = [
  ["/site/", join(REPO, "site")],
  ["/", join(REPO, "e2e", "fixtures")],
];

/** A running page server. */
export interface PageServer {
  /** `http://localhost:<port>`, no trailing slash. */
  readonly origin: string;
  close(): Promise<void>;
}

/** Starts a server on a free port. */
export async function startServer(): Promise<PageServer> {
  const server = createServer(async (request, response) => {
    const file = resolvePath(new URL(request.url ?? "/", "http://localhost").pathname);
    const body = file === undefined ? undefined : await readFile(file).catch(() => undefined);
    if (file === undefined || body === undefined) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, {
      "content-type": TYPES[extname(file)] ?? "application/octet-stream",
      "cache-control": "no-store",
    });
    response.end(body);
  });
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  const { port } = server.address() as AddressInfo;
  return {
    origin: `http://localhost:${port}`,
    close: () => new Promise<void>((done) => server.close(() => done())),
  };
}

/** The file a URL path names, never outside its root. */
function resolvePath(path: string): string | undefined {
  const wanted = path === "/" ? "/page.html" : decodeURIComponent(path);
  for (const [prefix, root] of ROOTS) {
    if (!wanted.startsWith(prefix)) continue;
    const file = normalize(join(root, wanted.slice(prefix.length)));
    return file.startsWith(root + sep) ? file : undefined;
  }
  return undefined;
}

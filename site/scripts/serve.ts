// Serves dist/ the way GitHub Pages does: /x is x.html, /x/ is x/index.html, a folder without
// its trailing slash is redirected to it, and anything missing gets 404.html with status 404.
// `bun run preview` serves it on port 4173; the end-to-end tests start it on a free port.

import { existsSync, readFileSync, statSync } from "node:fs";
import { createServer, type Server } from "node:http";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

export const distDir = fileURLToPath(new URL("../dist", import.meta.url));

const TYPES: Readonly<Record<string, string>> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".woff2": "font/woff2",
  ".xml": "application/xml",
  ".txt": "text/plain; charset=utf-8",
};

const isFile = (path: string) => existsSync(path) && statSync(path).isFile();

/** The file a URL path names under Pages' rules, or a redirect, or nothing. */
export function resolve(
  root: string,
  path: string,
): { file: string } | { redirect: string } | undefined {
  const target = normalize(join(root, path));
  if (!target.startsWith(root)) return undefined;
  if (isFile(target)) return { file: target };
  if (existsSync(target) && statSync(target).isDirectory()) {
    if (!path.endsWith("/")) return { redirect: `${path}/` };
    if (isFile(join(target, "index.html"))) return { file: join(target, "index.html") };
  }
  if (!path.endsWith("/") && isFile(`${target}.html`)) return { file: `${target}.html` };
  return undefined;
}

export function serve(port = 0, root = distDir): Promise<{ url: string; server: Server }> {
  const server = createServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    const found = resolve(root, path);
    if (found && "redirect" in found) {
      response.writeHead(301, { Location: found.redirect }).end();
      return;
    }
    const file = found?.file ?? join(root, "404.html");
    const type = TYPES[extname(file)] ?? "application/octet-stream";
    const body = readFileSync(file);
    // GitHub Pages compresses text; so does the preview, so what is measured is what ships.
    const compress =
      /^(text|application\/(json|xml))|svg/.test(type) &&
      /\bgzip\b/.test(request.headers["accept-encoding"] ?? "");
    response.writeHead(found ? 200 : 404, {
      "Content-Type": type,
      "Cache-Control": "no-store",
      ...(compress ? { "Content-Encoding": "gzip", Vary: "Accept-Encoding" } : {}),
    });
    response.end(compress ? gzipSync(body) : body);
  });
  return new Promise((done) => {
    server.listen(port, "127.0.0.1", () => {
      const address = server.address();
      const actual = typeof address === "object" && address ? address.port : port;
      done({ url: `http://localhost:${actual}`, server });
    });
  });
}

if (import.meta.main) {
  const { url } = await serve(Number(process.env.PORT ?? 4173));
  console.log(`Serving dist/ at ${url}`);
}

// Serves page.html on a random localhost port: the extension only injects
// itself into http://localhost and http://127.0.0.1 pages.

import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const PAGE = join(dirname(fileURLToPath(import.meta.url)), "..", "page.html");

/** Starts the server; resolves to `{ url, close }`. */
export async function startPageServer() {
  const server = createServer(async (request, response) => {
    if (request.url === "/") {
      response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
      response.end(await readFile(PAGE));
    } else {
      response.writeHead(204).end();
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  return {
    url: `http://127.0.0.1:${port}/`,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

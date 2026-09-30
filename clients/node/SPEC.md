# @websign/desktop — specification

Surface: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md)
§7. Tests use a fake `websign connect` (a Node script speaking the protocol
from a scenario file).

## 1. `findExecutable()`

`WEBSIGN_EXECUTABLE` if it exists; else `websign`/`websign.exe` on `PATH`;
else, in order: Windows `%LOCALAPPDATA%\Programs\WebeSign\websign.exe`,
`%LOCALAPPDATA%\Microsoft\WindowsApps\websign.exe`; macOS
`/Applications/WebeSign.app/Contents/MacOS/websign`,
`~/Applications/WebeSign.app/Contents/MacOS/websign`; Linux
`/usr/bin/websign`, `~/.local/bin/websign`. `undefined` when none exists.

## 2. Connection and framing

- `connect()`: spawn `[exe, "connect"]` with `stdio: ["pipe","pipe","ignore"]`;
  spawn error or immediate exit → `AppMissing`. Send `hello` with
  `client {name:"@websign/desktop", version}` and `protocols {min:1,max:1}`;
  wait ≤ 10 s for the `hello` reply; `error` → reject with its code.
- Frames: `u32` little-endian + UTF-8 JSON; `FrameDecoder` handles frames
  split across chunks and several frames per chunk; > 1 MiB → connection
  error.
- Ids: `"n" + counter`.
- Child exit with requests open → reject them with `Internal`
  ("the app exited").
- `close()`: end stdin, wait for exit (≤ 5 s, then kill).

## 3. Methods

Same flows as the SDK (§5–§6 of `sdk/SPEC.md`) without `web` and without
discovery: `status`, `certificates(filter)`, `sign(options)` (with
`prepare`, stale-seq handling, `cancel` on throw or abort),
`openDiagnostics(tab)` → `done`. Results decode Base64 to `Uint8Array`.

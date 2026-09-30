//! The examples and notes `--help` prints after the flags: what a caller
//! needs to use a command correctly the first time (`docs/architecture/desktop-api.md`).

/// After the command list.
pub const MAIN: &str = "\
Without a command, opens the diagnostics window.

Machine-facing output (sign, choose, connect, --json) goes to stdout as one
JSON document; messages for people go to stderr.

Exit codes: 0 success, 1 internal error, 2 usage error, 3-15 the protocol's
error codes (cancelled, timeout, no certificates…; see
docs/architecture/desktop-api.md §4).";

/// `websign sign --help`.
pub const SIGN: &str = "\
Examples:
  # A SHA-256 digest in hex; the window asks the person to confirm.
  websign sign --hash SHA-256 \\
    --digest 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

  # Raw digest bytes from another program, ECDSA preferred.
  openssl dgst -sha256 -binary invoice.xml \\
    | websign sign --hash SHA-256 --digest-file - --algorithm ECDSA

  # With the certificate picked earlier by `websign choose`.
  websign sign --hash SHA-384 --digest-file digest.bin --certificate <SHA-256 FINGERPRINT>

Output (stdout, one line):
  {\"type\":\"sign.result\",\"hash\":\"SHA-256\",\"algorithm\":\"ECDSA\",\"signature\":\"…\",\"certificate\":{…}}
  {\"type\":\"error\",\"code\":\"UserCancelled\",\"message\":\"…\"}   (exit code 3)

For digests that depend on the certificate (PAdES, CAdES), use `websign
connect` or a client library instead.";

/// `websign choose --help`.
pub const CHOOSE: &str = "\
Examples:
  websign choose
  websign choose --algorithm ECDSA --algorithm RSASSA-PSS

Output (stdout, one line):
  {\"type\":\"choose.result\",\"certificates\":[{…}]}
  {\"type\":\"error\",\"code\":\"NoCertificates\",\"message\":\"…\"}   (exit code 5)";

/// `websign connect --help`.
pub const CONNECT: &str = "\
The program that runs this command is the caller the window shows (its
name and code signature, never anything it sends).

Frames, both ways: a 32-bit little-endian length, then that many bytes of
UTF-8 JSON. Send `hello` first, then requests (docs/architecture/protocol.md).
Closing stdin ends the process; it also exits after 300 s without a request.

Example (Python):
  import json, struct, subprocess
  app = subprocess.Popen([\"websign\", \"connect\"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
  def send(message):
      data = json.dumps(message).encode()
      app.stdin.write(struct.pack(\"<I\", len(data)) + data); app.stdin.flush()
  def receive():
      (size,) = struct.unpack(\"<I\", app.stdout.read(4))
      return json.loads(app.stdout.read(size))

Client libraries: @websign/desktop (Node, Electron), websign-client (Rust).";

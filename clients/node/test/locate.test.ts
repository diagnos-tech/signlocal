import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { type Environment, search } from "../src/candidates";
import { findExecutable } from "../src/locate";

function environment(
  platform: NodeJS.Platform,
  env: Record<string, string>,
  existing: string[],
  home = "/home/ana",
): Environment {
  return { platform, env, home, isExecutable: (path) => existing.includes(path) };
}

describe("search order", () => {
  it("prefers SIGNLOCAL_EXECUTABLE over PATH and install locations", () => {
    const e = environment("linux", { SIGNLOCAL_EXECUTABLE: "/opt/ws", PATH: "/bin" }, [
      "/opt/ws",
      "/bin/signlocal",
      "/usr/bin/signlocal",
    ]);
    expect(search(e)).toBe("/opt/ws");
  });

  it("ignores a SIGNLOCAL_EXECUTABLE that does not exist", () => {
    const e = environment("linux", { SIGNLOCAL_EXECUTABLE: "/nope", PATH: "/bin" }, [
      "/bin/signlocal",
    ]);
    expect(search(e)).toBe("/bin/signlocal");
  });

  it("prefers PATH over install locations, in PATH order", () => {
    const e = environment("linux", { PATH: "/a::/b" }, [
      "/b/signlocal",
      "/a/signlocal",
      "/usr/bin/signlocal",
    ]);
    expect(search(e)).toBe("/a/signlocal");
  });

  it("falls back to /usr/bin then ~/.local/bin on Linux", () => {
    const both = ["/usr/bin/signlocal", "/home/ana/.local/bin/signlocal"];
    expect(search(environment("linux", {}, both))).toBe("/usr/bin/signlocal");
    expect(search(environment("linux", {}, both.slice(1)))).toBe("/home/ana/.local/bin/signlocal");
  });

  it("falls back to the system then the user app bundle on macOS", () => {
    const system = "/Applications/signlocal.app/Contents/MacOS/signlocal";
    const user = "/Users/ana/Applications/signlocal.app/Contents/MacOS/signlocal";
    expect(search(environment("darwin", {}, [system, user], "/Users/ana"))).toBe(system);
    expect(search(environment("darwin", {}, [user], "/Users/ana"))).toBe(user);
  });

  it("falls back to the ~/.local/bin symlink last on macOS", () => {
    const user = "/Users/ana/Applications/signlocal.app/Contents/MacOS/signlocal";
    const link = "/Users/ana/.local/bin/signlocal";
    expect(search(environment("darwin", {}, [link, user], "/Users/ana"))).toBe(user);
    expect(search(environment("darwin", {}, [link], "/Users/ana"))).toBe(link);
  });

  it("searches PATH then both Windows locations, with backslashes", () => {
    const local = "C:\\Users\\Ana\\AppData\\Local";
    const programs = `${local}\\Programs\\signlocal\\signlocal.exe`;
    const alias = `${local}\\Microsoft\\WindowsApps\\signlocal.exe`;
    const env = { LOCALAPPDATA: local };
    expect(search(environment("win32", env, [programs, alias]))).toBe(programs);
    expect(search(environment("win32", env, [alias]))).toBe(alias);
    expect(
      search(
        environment("win32", { ...env, Path: '"C:\\Tools";C:\\Bin' }, [
          "C:\\Bin\\signlocal.exe",
          alias,
        ]),
      ),
    ).toBe("C:\\Bin\\signlocal.exe");
  });

  it("skips Windows locations when LOCALAPPDATA is unset", () => {
    expect(search(environment("win32", {}, []))).toBeUndefined();
  });

  it("returns undefined when nothing exists", () => {
    expect(search(environment("linux", { PATH: "/bin" }, []))).toBeUndefined();
  });
});

describe("findExecutable", () => {
  const directories: string[] = [];
  afterEach(() => {
    delete process.env.SIGNLOCAL_EXECUTABLE;
    for (const d of directories.splice(0)) rmSync(d, { recursive: true, force: true });
  });

  it("honours SIGNLOCAL_EXECUTABLE on the real file system", () => {
    const directory = mkdtempSync(join(tmpdir(), "signlocal-locate-"));
    directories.push(directory);
    const file = join(directory, "signlocal");
    writeFileSync(file, "#!/bin/sh\n");
    chmodSync(file, 0o755);
    process.env.SIGNLOCAL_EXECUTABLE = file;
    expect(findExecutable()).toBe(file);
  });

  it("does not accept a directory or a non-executable file", () => {
    const directory = mkdtempSync(join(tmpdir(), "signlocal-locate-"));
    directories.push(directory);
    const plain = join(directory, "plain");
    writeFileSync(plain, "");
    chmodSync(plain, 0o644);
    for (const candidate of [directory, plain]) {
      process.env.SIGNLOCAL_EXECUTABLE = candidate;
      expect(findExecutable()).not.toBe(candidate);
    }
  });
});

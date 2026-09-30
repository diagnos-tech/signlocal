//! Programs that run other people's code: interpreters, shells and terminal
//! hosts.
//!
//! The app identifies a desktop caller by its parent process. When that
//! parent is `node`, `python` or `bash`, the process that asks is only the
//! runtime: every script it runs looks the same to the OS. Remembering it
//! would let any script on the computer receive the person's certificates
//! without a window, so these callers are never remembered, and the window
//! says the request comes from "a script run by {program}".

use std::path::Path;

/// Executable names, lowercase, without `.exe` and without a trailing
/// version (`python3.12` → `python`, `ruby3` → `ruby`).
const SCRIPT_HOSTS: &[&str] = &[
    // JavaScript and TypeScript runtimes.
    "node",
    "nodejs",
    "deno",
    "bun",
    // Python, including the Windows launcher and the windowless variants.
    "python",
    "pythonw",
    "py",
    "pyw",
    "pypy",
    // Other interpreters.
    "ruby",
    "rubyw",
    "irb",
    "perl",
    "php",
    "lua",
    "luajit",
    "tclsh",
    "wish",
    "rscript",
    "julia",
    "java",
    "javaw",
    "jshell",
    "dotnet",
    "osascript",
    "wscript",
    "cscript",
    "mshta",
    // Shells.
    "sh",
    "bash",
    "zsh",
    "dash",
    "ksh",
    "mksh",
    "csh",
    "tcsh",
    "fish",
    "nu",
    "xonsh",
    "elvish",
    "busybox",
    "env",
    "cmd",
    "command",
    "powershell",
    "powershell_ise",
    "pwsh",
    // Terminal hosts, which a script may inherit as its parent.
    "windowsterminal",
    "wt",
    "openconsole",
    "conhost",
    "terminal",
    "iterm2",
    "gnome-terminal-server",
    "konsole",
    "xterm",
    "alacritty",
    "kitty",
    "wezterm-gui",
];

/// Whether `executable` is an interpreter, shell or terminal host.
pub(super) fn is_script_host(executable: &Path) -> bool {
    let Some(name) = executable.file_name() else {
        return false;
    };
    let name = name.to_string_lossy().to_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);
    let base = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    SCRIPT_HOSTS.contains(&name) || (!base.is_empty() && SCRIPT_HOSTS.contains(&base))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(path: &str) -> bool {
        is_script_host(Path::new(path))
    }

    #[test]
    fn interpreters_and_shells_are_script_hosts() {
        for path in [
            "/usr/bin/node",
            "/usr/bin/python3",
            "/usr/local/bin/python3.12",
            "/usr/bin/ruby3.2",
            "/usr/bin/perl5.36",
            "/bin/bash",
            "/bin/sh",
            "/usr/bin/zsh",
            "/usr/bin/fish",
            "/opt/homebrew/bin/deno",
            "/home/a/.bun/bin/bun",
            "/usr/bin/osascript",
            "/usr/lib/jvm/bin/java",
        ] {
            assert!(host(path), "{path}");
        }
    }

    #[test]
    fn windows_names_match_without_case_or_extension() {
        for name in [
            "node.exe",
            "PYTHON.EXE",
            "pythonw.exe",
            "py.exe",
            "powershell.exe",
            "pwsh.exe",
            "cmd.exe",
            "WindowsTerminal.exe",
            "OpenConsole.exe",
            "conhost.exe",
            "wt.exe",
            "cscript.exe",
        ] {
            assert!(host(name), "{name}");
        }
    }

    #[test]
    fn ordinary_programs_are_not_script_hosts() {
        for path in [
            "/opt/acme/bin/invoicer",
            "/usr/bin/nodes",
            "/usr/bin/shell-tool",
            "/usr/bin/7z",
            "Acme.exe",
            "/",
            "",
        ] {
            assert!(!host(path), "{path}");
        }
    }
}

//! "Open diagnostics" from a connection: a separate `websign diagnostics`
//! process, so the window survives the browser closing the port (which ends
//! this host process).

use std::process::{Command, Stdio};

use websign_host::ports::Launcher;
use websign_protocol::messages::DiagnosticsTab;

/// Starts `websign diagnostics [--tab …]` detached from this process.
#[derive(Debug)]
pub struct ProcessLauncher;

impl Launcher for ProcessLauncher {
    fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| format!("current_exe: {:?}", e.kind()))?;
        let mut command = Command::new(exe);
        command.arg("diagnostics");
        if let Some(tab) = tab {
            command.args(["--tab", tab_arg(tab)]);
        }
        // stdin/stdout are the protocol pipes: the child must not inherit them.
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child =
            spawn_detached(&mut command).map_err(|e| format!("spawn: {:?}", e.kind()))?;
        log::info!("diagnostics started in a separate process");
        // Reaped on a thread so it never lingers as a zombie while we live.
        std::thread::spawn(move || child.wait());
        Ok(())
    }
}

/// The `--tab` value of `websign diagnostics`.
pub fn tab_arg(tab: DiagnosticsTab) -> &'static str {
    match tab {
        DiagnosticsTab::Browsers => "browsers",
        DiagnosticsTab::Devices => "devices",
        DiagnosticsTab::Certificates => "certificates",
        DiagnosticsTab::Help => "help",
    }
}

/// Unix: its own process group, so a signal to the browser's group spares it.
#[cfg(unix)]
fn spawn_detached(command: &mut Command) -> std::io::Result<std::process::Child> {
    use std::os::unix::process::CommandExt;
    command.process_group(0).spawn()
}

/// Windows: out of the browser's job object when the job allows it (a job
/// that kills its processes on close would take the window with the host),
/// else inside it.
#[cfg(windows)]
fn spawn_detached(command: &mut Command) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
    command
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB)
        .spawn()
        .or_else(|_| command.creation_flags(CREATE_NEW_PROCESS_GROUP).spawn())
}

#[cfg(not(any(unix, windows)))]
fn spawn_detached(command: &mut Command) -> std::io::Result<std::process::Child> {
    command.spawn()
}

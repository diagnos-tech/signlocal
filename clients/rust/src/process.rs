//! Starting `websign connect` and making sure it never outlives its caller.

use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::ClientError;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// `CREATE_NO_WINDOW`: keep a console flash away from GUI callers.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The child with the pipes the protocol runs over.
pub(crate) struct Spawned {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}

/// Runs `executable connect`. stderr is discarded: the app logs to its own
/// file, and an unread pipe could fill up and block it. Errors never name
/// the path: it usually contains the user's name.
pub(crate) fn spawn(executable: &Path, grace: Duration) -> Result<Spawned, ClientError> {
    let mut command = Command::new(executable);
    command
        .arg("connect")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .spawn()
        .map_err(|error| ClientError::AppMissing(format!("cannot start the app: {error}")))?;
    match (child.stdin.take(), child.stdout.take()) {
        (Some(stdin), Some(stdout)) => Ok(Spawned {
            child,
            stdin,
            stdout,
        }),
        _ => {
            reap(&mut child, grace);
            Err(ClientError::AppMissing(
                "the app's pipes are missing".into(),
            ))
        }
    }
}

/// Waits up to `grace` for the child to exit, then kills it, and always
/// collects its exit status so no zombie is left behind.
pub(crate) fn reap(child: &mut Child, grace: Duration) {
    let deadline = Instant::now() + grace;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            // Still running past the grace period, or the status is
            // unreadable: killing is the only way to be sure it ends.
            Ok(None) | Err(_) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

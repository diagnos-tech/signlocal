//! Running a desktop helper program without ever hanging the app.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Settings tools answer in milliseconds; one stuck on a dead D-Bus session
/// must not delay the window.
const DEADLINE: Duration = Duration::from_millis(800);
const POLL: Duration = Duration::from_millis(10);

/// The standard output of `program args` when it exits successfully within
/// [`DEADLINE`]; `None` when it is missing, fails, or is too slow (then it
/// is killed). Meant for short answers: a child that fills the pipe buffer
/// would block until the deadline.
pub fn output(program: &str, args: &[&str]) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + DEADLINE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                let mut text = String::new();
                child.stdout.take()?.read_to_string(&mut text).ok()?;
                return Some(text);
            }
            Ok(Some(_)) => return None,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
            _ => {
                stop(&mut child);
                return None;
            }
        }
    }
}

/// Starts `program args` without waiting for it; `after_exit` runs on a
/// background thread once it ends (or never, if the app exits first).
/// `false` when the program could not be started.
pub fn launch(program: &str, args: &[&str], after_exit: impl FnOnce() + Send + 'static) -> bool {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        return false;
    };
    let reaper = std::thread::Builder::new()
        .name("reap-helper".into())
        .spawn(move || {
            let _ = child.wait();
            after_exit();
        });
    // Without the thread the child is only reaped when the app exits.
    reaper.is_ok()
}

fn stop(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_returns_what_the_program_printed() {
        assert_eq!(output("echo", &["hello"]).as_deref(), Some("hello\n"));
    }

    #[test]
    fn output_is_none_for_missing_failing_or_slow_programs() {
        assert_eq!(output("websign-no-such-program", &[]), None);
        assert_eq!(output("false", &[]), None);
        let started = Instant::now();
        assert_eq!(output("sleep", &["5"]), None);
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn launch_runs_the_follow_up_after_exit() {
        let (done, finished) = std::sync::mpsc::channel();
        assert!(launch("true", &[], move || {
            let _ = done.send(());
        }));
        assert!(finished.recv_timeout(Duration::from_secs(5)).is_ok());
        assert!(!launch("websign-no-such-program", &[], || {}));
    }
}

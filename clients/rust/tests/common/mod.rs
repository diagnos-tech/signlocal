//! Shared helpers: the fake `websign connect` and a way to start it.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use websign_client::{Client, ClientError, ConnectOptions};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub const QUICK_HELLO: Duration = Duration::from_millis(1500);
pub const QUICK_GRACE: Duration = Duration::from_millis(500);

/// The fake app under a scenario name, removed on drop.
pub struct Fake {
    dir: PathBuf,
    pub executable: PathBuf,
}

impl Fake {
    /// Links the `fake_websign` example as `fake-<scenario>`: the fake reads
    /// its scenario from its own file name. Cargo builds examples before it
    /// runs integration tests, next to the `deps` directory.
    pub fn new(scenario: &str) -> Fake {
        let test_exe = std::env::current_exe().unwrap();
        let profile_dir = test_exe.parent().unwrap().parent().unwrap();
        let example = profile_dir
            .join("examples")
            .join(format!("fake_websign{}", std::env::consts::EXE_SUFFIX));
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "websign-client-fake-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let executable = dir.join(format!("fake-{scenario}{}", std::env::consts::EXE_SUFFIX));
        if fs::hard_link(&example, &executable).is_err() {
            fs::copy(&example, &executable).unwrap();
        }
        Fake { dir, executable }
    }

    pub fn options(&self) -> ConnectOptions {
        ConnectOptions {
            executable: Some(self.executable.clone()),
            ..ConnectOptions::default()
        }
    }

    pub fn connect(&self) -> Result<Client, ClientError> {
        Client::connect_with(self.options())
    }

    /// Connects with the waits shortened to `QUICK_HELLO` and `QUICK_GRACE`,
    /// for the scenarios that exercise them.
    pub fn connect_quickly(&self) -> Result<Client, ClientError> {
        websign_client::__testing::connect_with_waits(self.options(), QUICK_HELLO, QUICK_GRACE)
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Connects to the fake running `scenario`, keeping it alive with the client.
pub fn connect(scenario: &str) -> (Fake, Client) {
    let fake = Fake::new(scenario);
    let client = fake.connect().unwrap();
    (fake, client)
}

//! `ui-probe --renderer auto|wgpu|glow --out shot.png`
//!
//! Opens the sample window, saves one screenshot, prints a `RESULT` line and
//! exits. Exit code 0 only when the PNG was written.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use ui_probe::WINDOW_SIZE;
use ui_probe::app::{Outcome, ProbeApp};
use ui_probe::renderer::{Backend, Choice, is_renderer_failure, run_with_fallback};

/// Command line.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Renderer to use; `auto` tries wgpu first, then glow.
    #[arg(long, value_enum, default_value = "auto")]
    renderer: Choice,
    /// Where to write the screenshot PNG.
    #[arg(long)]
    out: PathBuf,
    /// Give up (exit 3) when no screenshot arrives within this time.
    #[arg(long, default_value_t = 30)]
    timeout_secs: u64,
}

fn native_options(backend: Backend) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer: backend.eframe(),
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_title("ui-probe"),
        ..Default::default()
    }
}

fn run_backend(backend: Backend, out: &Path, outcome: &Arc<Mutex<Outcome>>) -> eframe::Result {
    *outcome.lock().expect("outcome lock") = Outcome::default();
    let (out, outcome) = (out.to_path_buf(), Arc::clone(outcome));
    eframe::run_native(
        "ui-probe",
        native_options(backend),
        Box::new(move |cc| Ok(Box::new(ProbeApp::new(cc, backend, out, outcome)))),
    )
}

fn main() -> ExitCode {
    let args = Args::parse();
    let timeout = Duration::from_secs(args.timeout_secs);
    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        println!("RESULT status=timeout");
        std::process::exit(3);
    });

    let outcome = Arc::new(Mutex::new(Outcome::default()));
    let result = run_with_fallback(
        args.renderer,
        |backend| run_backend(backend, &args.out, &outcome),
        is_renderer_failure,
        |backend, error| eprintln!("{backend} failed ({error}); falling back"),
    );
    let outcome = outcome.lock().expect("outcome lock");
    let used = outcome.used.as_ref();
    let renderer = used.map_or_else(|| "none".to_owned(), |u| u.backend.to_string());
    let adapter = used.map_or("-", |u| u.description.as_str());
    match result {
        Ok(_) if outcome.saved => {
            println!(
                "RESULT status=ok renderer={renderer} adapter=\"{adapter}\" png={}",
                args.out.display()
            );
            ExitCode::SUCCESS
        }
        Ok(_) => {
            let why = outcome
                .error
                .as_deref()
                .unwrap_or("window closed before screenshot");
            println!(
                "RESULT status=fail renderer={renderer} adapter=\"{adapter}\" error=\"{why}\""
            );
            ExitCode::FAILURE
        }
        Err(error) => {
            println!("RESULT status=fail renderer={renderer} error=\"{error}\"");
            ExitCode::FAILURE
        }
    }
}

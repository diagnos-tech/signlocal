//! The monitor thread: one PC/SC context at a time, re-established when the
//! service goes away.

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::time::Duration;

use pcsc::{Context, Error, PNP_NOTIFICATION, ReaderState, Scope, State};

use super::tracker::{Observed, Tracker};
use super::{ContextSlot, DeviceEvent};
use crate::pcsc::hex_upper;

/// `SCardGetStatusChange` timeout. An infinite wait misses PnP events on some
/// pcsc-lite versions, and the periodic wake-up also re-lists the readers
/// where the PnP pseudo-reader is unsupported.
const WAIT: Duration = Duration::from_secs(5);
/// Pause before looking for a PC/SC service that is not running.
const RETRY: Duration = Duration::from_secs(3);
/// Pause after a transient error, so a persistent one cannot spin the CPU.
const BACKOFF: Duration = Duration::from_millis(500);

enum Outcome {
    Stop,
    Outage,
}

pub(super) fn run(events: Sender<DeviceEvent>, stop: Receiver<()>, slot: &ContextSlot) {
    let mut service_running: Option<bool> = None;
    let mut tracker = Tracker::new();
    loop {
        if let Ok(context) = Context::establish(Scope::User) {
            if service_running == Some(false) {
                if events
                    .send(DeviceEvent::ServiceChanged { running: true })
                    .is_err()
                {
                    return;
                }
                tracker = Tracker::after_outage();
            }
            service_running = Some(true);
            slot.set(Some(context.clone()));
            let outcome = watch(&context, &events, &stop, &mut tracker);
            slot.set(None);
            if matches!(outcome, Outcome::Stop) {
                return;
            }
        }
        if service_running != Some(false) {
            service_running = Some(false);
            if events
                .send(DeviceEvent::ServiceChanged { running: false })
                .is_err()
            {
                return;
            }
        }
        if pause(&stop, RETRY) {
            return;
        }
    }
}

fn watch(
    context: &Context,
    events: &Sender<DeviceEvent>,
    stop: &Receiver<()>,
    tracker: &mut Tracker,
) -> Outcome {
    let mut states = Vec::new();
    if pnp_supported(context) {
        states.push(ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE));
    }
    loop {
        if !matches!(stop.try_recv(), Err(TryRecvError::Empty)) {
            return Outcome::Stop;
        }
        if sync_readers(context, &mut states).is_err() {
            return Outcome::Outage;
        }
        // Nothing to wait on (no PnP, no reader): some services return at
        // once for an empty list, which would spin.
        if states.is_empty() {
            if !send_all(events, tracker.observe(Vec::new())) || pause(stop, WAIT) {
                return Outcome::Stop;
            }
            continue;
        }
        match context.get_status_change(WAIT, &mut states) {
            Ok(()) | Err(Error::Timeout) => {}
            Err(Error::Cancelled) => return Outcome::Stop,
            Err(Error::UnknownReader | Error::ReaderUnavailable) => {
                if pause(stop, BACKOFF) {
                    return Outcome::Stop;
                }
                continue;
            }
            Err(_) => return Outcome::Outage,
        }
        if !send_all(events, tracker.observe(observed(&states))) {
            return Outcome::Stop;
        }
        states.iter_mut().for_each(ReaderState::sync_current_state);
    }
}

/// Whether the service knows the PnP pseudo-reader. Where it does not, every
/// wait would fail at once with `UnknownReader` and no reader change would
/// ever be seen; the 5 s timeout re-list is then the only way to notice them.
fn pnp_supported(context: &Context) -> bool {
    let mut probe = [ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE)];
    !matches!(
        context.get_status_change(Duration::ZERO, &mut probe),
        Err(Error::UnknownReader)
    )
}

fn is_pnp(state: &ReaderState) -> bool {
    state.name() == PNP_NOTIFICATION()
}

/// Keeps the PnP pseudo-reader, if watched, and one state per listed reader.
fn sync_readers(context: &Context, states: &mut Vec<ReaderState>) -> Result<(), Error> {
    let names = match context.list_readers_owned() {
        Ok(names) => names,
        Err(Error::NoReadersAvailable) => Vec::new(),
        Err(error) => return Err(error),
    };
    states
        .retain(|state| is_pnp(state) || names.iter().any(|name| name.as_c_str() == state.name()));
    for name in names {
        if !states.iter().any(|state| state.name() == name.as_c_str()) {
            states.push(ReaderState::new(name, State::UNAWARE));
        }
    }
    Ok(())
}

fn observed(states: &[ReaderState]) -> Vec<Observed> {
    states
        .iter()
        .filter(|state| !is_pnp(state))
        .filter(|state| {
            !state
                .event_state()
                .intersects(State::UNKNOWN | State::IGNORE)
        })
        .map(|state| {
            let present = state.event_state().contains(State::PRESENT);
            Observed {
                name: state.name().to_string_lossy().into_owned(),
                card_present: present,
                atr: (present && !state.atr().is_empty()).then(|| hex_upper(state.atr())),
            }
        })
        .collect()
}

/// False when the receiver hung up.
fn send_all(events: &Sender<DeviceEvent>, batch: Vec<DeviceEvent>) -> bool {
    batch.into_iter().all(|event| events.send(event).is_ok())
}

/// Sleeps up to `duration`; true when the monitor was asked to stop meanwhile.
fn pause(stop: &Receiver<()>, duration: Duration) -> bool {
    !matches!(stop.recv_timeout(duration), Err(RecvTimeoutError::Timeout))
}

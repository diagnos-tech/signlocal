//! Kittest checks of the confirmation window: every state in light and dark
//! (snapshots), the anti-accident rules (arming, keystrokes before arming,
//! Enter, Esc), AccessKit (roles, names, the PIN's missing value, focus
//! order) and the time to the first frame.

mod fixtures;
mod scenes;
mod support;

mod accesskit;
mod arming;
mod callers;
mod engine_end;
mod keyboard;
mod open_time;
mod paths;
mod pin_fit;
mod snapshots;

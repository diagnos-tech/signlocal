//! Kittest checks of the diagnostics window: snapshots of every tab in both
//! themes, AccessKit roles and keyboard, the golden "Copy diagnostics" text,
//! the revoke flow against a consent store on disk, and "Add driver…"
//! through the file picker.

mod fixture;
mod support;

mod accesskit;
mod drivers;
mod golden;
mod revoke;
mod snapshots;

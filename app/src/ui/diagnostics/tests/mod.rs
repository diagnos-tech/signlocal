//! Kittest checks of the diagnostics window: snapshots of every tab in both
//! themes, AccessKit roles and keyboard, the golden "Copy diagnostics" text,
//! the revoke flow against a consent store on disk, "Add driver…" through
//! the file picker, "Getting started" and the whole AccessKit tree.

mod fixture;
mod support;

mod accesskit;
mod drivers;
mod getting_started;
mod golden;
mod revoke;
mod snapshots;
mod tree;

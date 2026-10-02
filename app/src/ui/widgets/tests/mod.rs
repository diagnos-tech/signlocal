//! Kittest checks of every widget: AccessKit roles, names and states, and a
//! snapshot per widget group in light and dark (`docs/architecture/testing.md`
//! §3). One file per widget group; [`support`] holds the harness.

mod support;

mod button;
mod cert_row;
mod code_card;
mod fields;
mod loading;
mod notices;
mod specimen;
mod tabs;

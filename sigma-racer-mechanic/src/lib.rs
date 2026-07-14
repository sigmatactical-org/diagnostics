//! Sigma Racer Mechanic library.

// deny (not forbid): the Slint-generated UI module carries its own
// scoped allow(unsafe_code) for vtable statics.
#![deny(unsafe_code)]

slint::include_modules!();

pub mod about;
pub mod app;
pub mod config;
pub mod state;

pub use state::AppState;

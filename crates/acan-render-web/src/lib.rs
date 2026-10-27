//! Browser host for `acan-render`. The plant input, the kind table, the layout and the stand-in
//! models build on every target, so `cargo test` covers them on the host. `App` is the wasm
//! surface that the page drives.

pub mod kind;
pub mod layout;
pub mod plant;
pub mod stand_in;

#[cfg(target_arch = "wasm32")]
mod app;

#[cfg(target_arch = "wasm32")]
pub use app::App;

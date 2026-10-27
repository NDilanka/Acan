//! The product screen's Rust side: the plant's plan, the origin's state folded from its log, and
//! the view the HTML panels draw.
//!
//! Everything but the canvas is plain Rust and runs natively, so the event-to-view mapping is
//! tested with `cargo test`. The wgpu canvas and the methods the page calls are wasm32 only.

pub mod layout;
pub mod mirror;
pub mod paint;
pub mod scenario;

#[cfg(target_arch = "wasm32")]
mod web;

//! Through the Solar System: an annotated first-person flight from the Sun
//! to the Oort cloud, drawn with WebGL2.
//!
//! The simulation (orbits, rotation, the catalogue of facts) is plain Rust
//! and is tested natively; the `web` module draws it and runs the game in a
//! browser.

pub mod astro;
pub mod belts;
pub mod camera;
pub mod catalog;
pub mod format;
pub mod mesh;
pub mod sim;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::start;

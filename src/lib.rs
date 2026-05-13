pub mod color;
pub mod game;
pub mod snake;
#[cfg(feature = "cli")]
pub mod ui;
#[cfg(feature = "wasm")]
pub mod web;

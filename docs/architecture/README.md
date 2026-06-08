# Architecture Overview

The `snake_game` is designed using a monolithic modular structure in Rust.

- **Entry Point:** `src/main.rs` initializes the game state, parses CLI arguments (such as `--bot` or `--spectator`), and runs the main event loop.
- **Library Root:** `src/lib.rs` exports all internal modules.
- **Game Engine:** `src/game/` handles all logic related to state progression, scoring, pathfinding, and interactions.
- **UI & Rendering:** `src/ui.rs` manages drawing elements on the terminal via the `crossterm` crate.

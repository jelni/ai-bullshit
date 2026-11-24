# Architecture Overview

The `snake_game` is designed using a monolithic modular structure in Rust.

- **Entry Point:** `src/main.rs` initializes the game state, parses CLI arguments (such as `--bot` or `--spectator`), and runs the main event loop.
- **Library Root:** `src/lib.rs` exports all internal modules.
- **Game Engine:** `src/game/` handles all logic related to state progression, scoring, pathfinding, and interactions.
- **UI & Rendering:** `src/ui.rs` manages drawing elements on the terminal via the `crossterm` crate.

## Autopilot Safety Pipeline

The autopilot (`Game::calculate_autopilot_move`, `calculate_p2_autopilot_move`, and the arena-bot branch of `handle_autopilot_moves` in `src/game/game_struct.rs`, which covers the arena bots of MassiveMultiplayer, Tron, Zombie, King of the Hill, and Turf War) decides every bot move through a layered pipeline:

1. **Target selection:** food, bonus food, power-ups, goblins, merchants, and mode-specific objectives (KoTH zone, CTF flags, dungeon doors/bosses, crops, and in Turf War the nearest frontier turf cells via `turf_war_frontier_targets`). Targets are filtered for contention first (`contested_targets`): a target is contested when a rival snake is predicted to reach it strictly earlier than the bot, either by wrap/portal-aware travel distance (`travel_dist`) or because the rival's committed head path crosses it sooner; contested targets are only tried when no uncontested target verifies (`astar_search_verified_contested`). Targets on a rival's current head (Zombie-style hunt targets) are never contested, and the filter is inert in single-player or while invincible.
2. **A\* search (`astar_search`):** heuristic penalties for hazards and enemy bodies, dynamic edge costs for lasers/mines/turrets/black holes/lightning/meteors, plus trap-aware tail-reachability validation (`path_keeps_tail_reachable`) so candidate paths that would seal the mover into a dead end are rejected during the search.
3. **Verified-safe selection (`astar_search_verified` + `path_is_safe`, driven by `astar_search_verified_contested` for objective targets):** every candidate path is replayed step-by-step; each cell is re-checked with `is_safe_final_p` at its true arrival step. Rival snakes are predicted two ways: `rival_head_lane_mask` projects each rival's head straight ahead for `RIVAL_HEAD_HORIZON` steps (and the A* expansion avoids those lanes), while `predicted_enemy_occupancy` simulates every other snake forward along its committed direction queue so predicted enemy head/body occupancy at each step is a hard constraint. Among all verified paths, the one leaving the most reachable free space after arrival (`reachable_space_after_path`, minus path length) is kept.
4. **Survival fallbacks:** tail-chasing (`astar_search_verified` toward the snake's own tail), then `flood_fill_fallback`, which picks the first step maximizing reachable open space.

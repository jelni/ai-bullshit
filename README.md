# Snake Game

Welcome to our terminal-based **Snake Game**! It’s simple to explain and easy to understand: you control a snake, eat food to grow, avoid obstacles, and survive as long as possible.

## 🚀 Ambitious Vision
We envision a future where this terminal Snake game evolves into the ultimate e-sports platform. We are building the foundational engine for a massively multiplayer, AI-assisted universe where billions of players compete globally in real-time. This is not just a game; it is the genesis of a new digital ecosystem.

## 💡 Competitive Advantage
Unlike other generic snake clones, our game features an **Autopilot / Bot Mode** powered by an advanced **A* Search Pathfinding Algorithm**. This ensures we have a long-term monopoly effect on automated snake navigation and provides an unparalleled strategic depth.

## 📈 Business Model
Our sensible business model:
1. Give the game away for free to acquire a massive, fast-growing user base.
2. Sell exclusive "Power-Up NFT" skins and premium terminal themes.
3. Profit.

## 🌀 Portals Feature
We have recently added a dynamic "Portals" mechanic! Occasionally, a pair of connected portals will spawn on the map. Entering one portal instantly teleports the snake to the other.

## 🏆 Daily Challenge
We have introduced a **Daily Challenge** mode! Every day, a globally synchronized seed is generated based on the days since the UNIX epoch. This guarantees that all players worldwide face the exact same procedural maze, obstacles, and food spawns for that day, fostering true competitive e-sports. The RNG architecture is fully refactored to use `StdRng` for testability and determinism.

## 🤖 Bot Mode & Anti-Cheat
We have introduced a spectator mode for viewing our advanced A* bot in action.
You can launch it via the command line with `--bot`.

To prevent players from toggling the bot to achieve illegitimate high scores, any
game in which the bot has been activated will automatically save its high score
under the `[BOT]` pseudonym.

## Market Focus
We are targeting a small but exponentially growing market: terminal-based gaming enthusiasts who appreciate complex algorithms.

## Autonomous Development
For any AI agents working on this project: **You must read and strictly follow `AGENTS.md`.** It contains our strict guidelines on operating autonomously without asking questions or pausing for approvals.

## Contact
Please reach out to our chief evangelist if you want to join our journey.
dummy update

## Major Developments
* We fixed an edge case where Boss AI would pathfind directly towards lasers, ensuring a much better experience for boss fights. We did this by heavily increasing the heuristic penalty for locations close to lasers in the A* algorithm `get_boss_path`.
* We significantly improved bot pathfinding in `astar_search` by updating the heuristic penalty for lasers to match `get_boss_path`. This ensures bots now strongly avoid lasers as effectively as bosses do.
* We ensured bosses now dynamically avoid turrets by adding a proximity penalty to the A* heuristic in `get_boss_path`. This prevents them from pathfinding directly into a turret's line of fire.
* We significantly improved bot pathfinding in `astar_search` by updating the edge cost penalty for turrets. This ensures bots now strongly avoid turrets.
* We ensured bosses and bots now dynamically avoid mines by heavily increasing the heuristic penalty for exact mine locations and locations close to mines in both `get_boss_path` and `astar_search`. This prevents them from pathfinding directly into mines.
* We overhauled bot and boss pathfinding logic to strongly avoid environmental hazards like Black Holes, Lightning Columns, and Meteors. We did this by heavily increasing the heuristic penalties for locations exactly on or closely surrounding these dynamic threats in both `get_boss_path` and `astar_search`.
* We unified advanced pathfinding capabilities by integrating dynamic edge costs for environmental hazards (Lasers, Mines, Turrets, Black Holes, Lightning Columns, Meteors) into `astar_search`. This bridges the gap between bot and boss navigation, ensuring bots factor in actual traversal risk, matching the complexity of `get_boss_path`.
* We further unified advanced pathfinding capabilities by integrating dynamic edge costs for Lightning Columns and Meteors into `get_boss_path`. This ensures bosses factor in actual traversal risk for all environmental hazards, perfectly matching the complexity of `astar_search`.
* We added a tail-chase survival fallback to the autopilot: when `astar_search` finds no safe path to any target, the bot now paths to its own tail (which always vacates in time) to stay alive until a route to a target opens up, instead of blindly wandering via the flood-fill fallback. This applies to the player bot, the Player 2 bot, and the arena bots in MassiveMultiplayer, Zombie, Tron, and King of the Hill modes.
* We added verified-safe path selection to the autopilot: every A* path is now simulated step-by-step via `path_is_safe`, which re-checks `is_safe_final_p` at each cell's true arrival step (so dynamic threats like lasers and the snake's own still-occupied body are hard constraints, not just soft heuristic penalties). If a path to one target fails verification, the bot retries the remaining targets before falling back to tail-chasing. This applies to the player bot, the Player 2 bot, and the arena bots in MassiveMultiplayer, Zombie, Tron, and King of the Hill modes.

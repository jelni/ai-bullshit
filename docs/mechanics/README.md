# Game Mechanics

The game features several complex mechanics beyond simple snake movement:

- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time). Verification also predicts rival snakes: each rival is assumed to keep moving straight along its current direction (through portals and wrapping) for up to `Game::RIVAL_HEAD_HORIZON` (8) steps, and any path cell that a rival's predicted head occupies at that cell's arrival step is rejected as a head-on collision; the A* expansion avoids these predicted lanes as well (see [tests/test_bot_rival_prediction.rs](../../tests/test_bot_rival_prediction.rs)). On top of that, `path_is_safe` simulates every other snake forward along its committed direction queue for up to 24 steps (`predicted_enemy_occupancy`) and treats any cell predicted to be occupied by an enemy head or body at a given step as a hard constraint, so queued turns and swept body lanes are avoided too. Paths are also checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists.
- **Flow-Field Autopilot (Massive Multiplayer / Zombie):** Arena bots in these modes normally follow a BFS flow field toward their targets for performance. Before a flow-field step is taken, the implied route is extracted with `follow_flow_field` and held to the same standards as the A* autopilot: every cell must pass `path_is_safe`, and `path_keeps_tail_reachable` must confirm the bot's tail stays reachable after simulating the route. If the flow-field route is unsafe or trapping, the bot falls back to the verified A* search, then tail-chasing, then the flood-fill survival move.
- **Power-ups:** Players can collect various power-ups that modify gameplay (e.g., speed boosts, invincibility, or score multipliers).
- **Daily Challenges:** Deterministic challenges generated using a specific global seed.
- **Missions/Bounties:** Dynamic quests spawned during the game.

## Weather System

The `weather` field on the game state cycles through `Clear`, `Rain`, `Snow`, `Storm`, `Tornado`, `Sandstorm`, `Earthquake`, `Eclipse` and `Fog` (randomly over time, and forced per-mode/per-planet in some cases). Each weather type has concrete gameplay effects, not just visuals:

- **Snow:** occasionally delays the autopilot bot (except in Blizzard mode, where the storm is permanent but the bot is never delayed).
- **Sandstorm:** occasionally shuffles the food to a neighboring cell.
- **Earthquake:** occasionally adds or removes an obstacle.
- **Tornado:** occasionally relocates the food.
- **Storm:** occasionally calls down a lightning column that devastates a whole row (including bosses standing in it).
- **Eclipse / Fog / Night:** restrict visibility to a radius around the snake(s).

The Druid hero class ignores all weather penalties.

## Blizzard Mode

`GameMode::Blizzard` (menu entry "Blizzard Mode") is a survival mode wrapped in a permanent snowstorm (`weather` is forced to `Weather::Snow` every tick):

- **Snowfall:** every 6th tick a snow drift (`*`) falls on a random empty cell, up to a cap of 10% of the board. Snow tiles are stored in `snow_tiles` and mirrored into `obstacles`, so they block movement for the player, the autopilot, and all arena bots like any other obstacle.
- **Fairness:** snow never falls on the snake's body, on any entity (food, power-ups, portals, mines, bosses, turrets, companion, goblin, ...), or on the cell any snake is about to move into this tick.
- **Thawing:** at tick 200 of every 250-tick cycle, all accumulated snow melts away ("Thawing!"), keeping the board playable.
- **Melting by food:** eating food melts all snow tiles adjacent to the eaten cell.
- **Persistence:** snow survives respawns and is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). If another effect clears obstacles (e.g. the Bomb power-up), stale snow records are pruned on the next tick.
- **Rendering:** snow drifts are drawn as white `*` (terminal) / white cells (web) on top of the red `X` obstacles, and the status bar shows a `Snow: <count>` counter.

## Turf War Mode

`GameMode::TurfWar` (menu entry "Turf War") is a territory-control mode: every snake (player, player 2, and the 3 arena bots, ids 3-5) paints each cell its body occupies into `painted_tiles` on every tick, overwriting whoever held it before. When `match_time` reaches 0, the participant with the most painted tiles wins ("Time's Up! Bot N Wins!" etc.).

- **Autopilot arena bots:** Turf War bots are driven by the same verified-safe autopilot as the other arena modes (Massive Multiplayer, Tron, Zombie, King of the Hill). Every move comes from `astar_search_verified`, so each candidate path is checked cell-by-cell with `path_is_safe` (including rival-head prediction and predicted enemy occupancy), must keep the tail reachable (`path_keeps_tail_reachable`), and is scored by reachable free space; bots fall back to tail-chasing and then the flood-fill survival move when no verified path exists. They also use `should_bot_shoot` like the other arena bots.
- **Frontier targeting:** before pathfinding, each bot prepends up to 3 targets from `turf_war_frontier_targets(start, bot_id, 3)` — the nearest in-bounds cells it has not painted itself yet (its current cell, own-painted cells, obstacles, and snake bodies are excluded). Enemy-owned tiles count as frontier, so bots actively contest and reclaim rival territory. Food, bonus food, power-ups, and the goblin remain in the target list after the frontier cells.
- **Player bot:** with `--bot`, `calculate_autopilot_move` also prepends up to 5 frontier cells (`turf_war_frontier_targets(start, 1, 5)`, i.e. cells the player has not painted itself) to its targets in this mode, so the player bot actively expands territory instead of only chasing food.
- **Scoring:** at timeout, painted tiles are tallied per owner id (1 = player 1, 2 = player 2, 3+ = bots) and all participants tied at the maximum score are declared winners.
- **Tests:** see [tests/test_turf_war_autopilot.rs](../../tests/test_turf_war_autopilot.rs) and [tests/test_turf_war.rs](../../tests/test_turf_war.rs).

# Game Mechanics

The game features several complex mechanics beyond simple snake movement:

- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time). Verification also predicts rival snakes: each rival is assumed to keep moving straight along its current direction (through portals and wrapping) for up to `Game::RIVAL_HEAD_HORIZON` (8) steps, and any path cell that a rival's predicted head occupies at that cell's arrival step is rejected as a head-on collision; the A* expansion avoids these predicted lanes as well (see [tests/test_bot_rival_prediction.rs](../../tests/test_bot_rival_prediction.rs)). On top of that, `path_is_safe` simulates every other snake forward along its committed direction queue for up to 24 steps (`predicted_enemy_occupancy`) and treats any cell predicted to be occupied by an enemy head or body at a given step as a hard constraint, so queued turns and swept body lanes are avoided too. Paths are also checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists. In multiplayer modes the autopilot also avoids contested targets (`contested_targets`): any food, power-up, or objective that a rival snake is predicted to reach strictly first (by wrap/portal-aware travel distance via `travel_dist`, or because the rival's committed head path crosses it sooner) is skipped in favor of an uncontested target, and only raced for when nothing else verifies (`astar_search_verified_contested`; see [tests/test_bot_contested_targets.rs](../../tests/test_bot_contested_targets.rs)).
- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time). Verification also predicts rival snakes: each rival is assumed to keep moving straight along its current direction (through portals and wrapping) for up to `Game::RIVAL_HEAD_HORIZON` (8) steps, and any path cell that a rival's predicted head occupies at that cell's arrival step is rejected as a head-on collision; the A* expansion avoids these predicted lanes as well (see [tests/test_bot_rival_prediction.rs](../../tests/test_bot_rival_prediction.rs)). On top of that, `path_is_safe` simulates every other snake forward along its committed direction queue for up to 24 steps (`predicted_enemy_occupancy`) and treats any cell predicted to be occupied by an enemy head or body at a given step as a hard constraint, so queued turns and swept body lanes are avoided too. Paths are also checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists.
- **Hunter AI (rival cut-off):** in modes with rival snakes (PlayerVsBot, BotVsBot, MassiveMultiplayer, Tron, King of the Hill, Capture the Flag, Zombie), a bot that is strictly longer than a nearby rival switches from pure survival to hunting: it projects each shorter rival's head up to `Game::CUTOFF_HORIZON` (12) steps ahead along its committed direction queue (`cutoff_targets`) and, if it can reach the cell where the rival's head will be after `k` steps within `k - 1` steps, it moves there so the rival crashes into its body on step `k` (collision checks use pre-move bodies, so the mover's body after `k - 1` moves is what kills). Kill paths must pass the same verified-safe, non-trapping checks as any other path, plus `path_cuts_off_rival`, which simulates the path against every shorter rival's predicted head path (the mover's own tail cell does not count as a kill, since it vacates in time). Among verified kill paths the shortest one is chosen; when no safe cut-off exists the bot falls back to its normal target selection. A CTF flag carrier never hunts. This applies to the player bot, the Player 2 bot, and the arena bots (see [tests/test_bot_cutoff_hunting.rs](../../tests/test_bot_cutoff_hunting.rs)).
- **Shrinking-zone awareness (Battle Royale):** in `GameMode::BattleRoyale` the safe zone shrinks by one cell per side every 10 s of wall-clock time (`safe_zone_margin`), and the engine treats the zone border exactly like a wall. The autopilot now enforces the same rule everywhere: `is_safe_final_p` rejects any cell outside `within_safe_zone(p, steps)` as a hard constraint (even while invincible, matching the engine), where the margin is projected `steps` ticks into the future by `zone_margin_at` (assuming the slowest plausible 500 ms tick, so the projection is conservative); `astar_search` additionally refuses to expand into out-of-zone cells even for target cells; and the trap/space flood fills (`path_keeps_tail_reachable`, `reachable_space_after_path`) only count cells inside the zone projected to the path's end. The flow-field autopilot inherits this through `is_safe_final_p` and `zone_margin_now`. Covered by [tests/test_bot_zone_aware.rs](../../tests/test_bot_zone_aware.rs).
- **Flow-Field Autopilot (Massive Multiplayer / Zombie):** Arena bots in these modes normally follow a BFS flow field toward their targets for performance. Before a flow-field step is taken, the implied route is extracted with `follow_flow_field` and held to the same standards as the A* autopilot: every cell must pass `path_is_safe`, and `path_keeps_tail_reachable` must confirm the bot's tail stays reachable after simulating the route. If the flow-field route is unsafe or trapping, the bot falls back to the verified A* search, then tail-chasing, then the flood-fill survival move.
- **Power-ups:** Players can collect various power-ups that modify gameplay (e.g., speed boosts, invincibility, or score multipliers).
- **Daily Challenges:** Deterministic challenges generated using a specific global seed.
- **Missions/Bounties:** Dynamic quests spawned during the game.

## Weather System

The `weather` field on the game state cycles through `Clear`, `Rain`, `Snow`, `Storm`, `Tornado`, `Sandstorm`, `Earthquake`, `Eclipse`, `Fog`, `Wildfire` and `Firestorm` (randomly over time for the ambient types, and forced per-mode/per-planet in some cases). Each weather type has concrete gameplay effects, not just visuals:

- **Rain:** raises the ambient bonus-food spawn chance. In Acid Rain Mode the rain is permanent and a downpour is guaranteed every 6th tick, dropping lethal acid puddles (see below).
- **Snow:** occasionally delays the autopilot bot (except in Blizzard mode, where the storm is permanent but the bot is never delayed).
- **Sandstorm:** occasionally shuffles the food to a neighboring cell. In Sandstorm Mode the storm is permanent, a sandblast is guaranteed every 6th tick, visibility is limited to a radius of 6, and sand drifts accumulate on the board (see below).
- **Earthquake:** occasionally adds or removes an obstacle. In Quake Mode the earthquake is permanent and a quake is guaranteed every 6th tick. In Earthquake Mode the earthquake is also permanent, with a fair scheduled tremor every 5th tick.
- **Earthquake:** occasionally adds or removes an obstacle. In Quake Mode the earthquake is permanent and a quake is guaranteed every 6th tick; in Seismic Mode a full row tremor is guaranteed every 6th tick.
- **Tornado:** occasionally relocates the food. In Tornado Mode the twister is permanent: a roaming vortex drifts every 6th tick and drags items towards itself, every 8th tick a board-wide gust shoves every snake one cell in a shared random direction, and every 6th tick a full twister spin reshuffles items and obstacles (see below).
- **Storm:** occasionally calls down a lightning column that devastates a whole row (including bosses standing in it). In Storm Mode the storm is permanent and a strike is guaranteed every 6th tick.
- **Eclipse / Fog / Night:** restrict visibility to a radius around the snake(s). In Eclipse Mode the eclipse is permanent and only solar flares (see below) lift the darkness.
- **Wildfire:** drifting embers are drawn while a wildfire rages. In Wildfire Mode the wildfire is permanent: cells ignite and spread every 6th tick (see below).

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

## Storm Mode

`GameMode::Storm` (menu entry "Storm Mode") is a survival mode wrapped in a permanent thunderstorm (`weather` is forced to `Weather::Storm` every tick):

- **Scheduled strikes:** on every 6th tick a lightning column is guaranteed to strike a random column (on top of the usual ambient storm strike chance). The strike devastates the whole row: snakes whose head enters the column die, bosses in it lose health, and all obstacles in that column are burned away.
- **Fairness:** a scheduled strike never targets the column a snake currently occupies or the column its head is about to enter this tick, so the storm is dangerous but never unfair.
- **Grounding:** eating food clears the currently active lightning column ("Grounded!"), giving the player a way to neutralize an incoming strike.
- **Autopilot:** the active lightning column is already a hard hazard in the A* heuristic/edge costs and in `is_safe_final_p` verified-safe path checks, so the autopilot routes around it natively.
- **Persistence:** the `storm_strikes` counter is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent), and the active `lightning_column` was already persisted. The status bar shows a `Strikes: <count>` counter, and the column is rendered as a yellow `|` (terminal) / yellow bar (web).

## Quake Mode

`GameMode::Quake` (menu entry "Quake Mode") is a survival mode wrapped in a permanent earthquake (`weather` is forced to `Weather::Earthquake` every tick, and the ambient weather randomization is skipped in this mode):

- **Scheduled quakes:** on every 6th tick a quake is guaranteed (on top of the ambient earthquake chance). A quake either shakes an existing obstacle loose (removed from `obstacles`) or drops a new rubble pile on a random empty cell, capped at 25% of the board so the arena stays playable.
- **Fairness:** rubble never falls on a snake's body, on the cell any snake is about to move into this tick, or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors).
- **Stabilizing:** eating food clears one random rubble pile ("Stabilized!"), giving the player a way to fight back against accumulating debris.
- **Autopilot:** obstacles are already hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance, so the autopilot routes around fresh rubble natively and rides out the quakes.
- **Persistence:** the `quake_count` counter is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows a `Quakes: <count>` counter, and rubble is rendered as a regular red `X` obstacle (terminal) / red cell (web).
- **Tests:** see [tests/test_quake_mode.rs](../../tests/test_quake_mode.rs).

## Volcano Mode

`GameMode::Volcano` (menu entry "Volcano Mode") is a survival mode built around a growing lava field:

- **Vents:** at reset, two volcano vents spawn on random empty cells (`volcano_vents`). Vent cells are lava and are permanent fixtures of the board.
- **Eruptions:** on every 6th tick a scheduled eruption (`erupt_volcano`) spreads lava from a random lava-frontier cell to a random orthogonal neighbor, up to `2 + vent_count` cells per eruption, capped at 15% of the board. Eruptions are counted in `volcano_eruptions` and shown as `Eruptions: <count>` in the status bar.
- **Lethality:** lava is stored in `lava_tiles` and mirrored into `obstacles`, so entering a lava tile kills the snake, and the autopilot treats lava as a hard obstacle natively (A* heuristic/edge costs, `is_safe_final_p` verified-safe path checks, and trap-avoidance flood fills all see it through `obstacles`).
- **Fairness:** lava never spreads onto a snake's body, the cell any snake is about to enter this tick, or any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, lasers, meteors, crops, eggs, flags, resources, ...).
- **Cooling cycle:** at tick 200 of every 250-tick cycle the whole lava field cools away ("The volcano cools!"), leaving only the vents, keeping the board playable.
- **Cooling by food:** eating food cools all lava tiles adjacent to the eaten cell ("Cooled N lava!").
- **Persistence:** `lava_tiles`, `volcano_vents` and `volcano_eruptions` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). If another effect clears obstacles (e.g. the Bomb power-up), stale lava records are pruned on the next tick and the vents are re-seated into `obstacles`.
- **Rendering:** lava is drawn as a red `~` (terminal) / orange cell (web) on top of the red `X` obstacles, vents as a dark-red `V` / dark-red cell, and the status bar shows `Lava: <count> | Eruptions: <count>`.
- **Tests:** see [tests/test_volcano_mode.rs](../../tests/test_volcano_mode.rs).

## Asteroids Mode

`GameMode::Asteroids` (menu entry "Asteroids Mode") is a survival mode under a permanent meteor storm:

- **Ambient meteor rain:** meteors spawn at the top of a random column with 15% chance per tick (vs. 1% in other modes) and fall one cell every 2 ticks.
- **Scheduled showers:** on every 12th tick (`tick_counter.is_multiple_of(12)`) a meteor shower fires: a burst of up to 3 meteors spawns in distinct columns at the top row. Each shower increments `meteor_showers`. The first meteor of a shower sweeps sideways (`vx = ±1`).
- **Drifting meteors:** every `Meteor` has a horizontal velocity `vx` (`#[serde(default)]`, so old saves load with `vx = 0`). While falling one cell every 2 ticks, a meteor with `vx != 0` also drifts one column per two fallen rows (on even rows) and bounces off the playable side walls (`meteor_drift_bounds`), sweeping diagonally across the board. Only the shower meteors above get `vx != 0`; all other meteors fall straight down.
- **Fairness:** in Asteroids mode, meteor spawn columns (ambient and shower alike) are filtered by `meteor_safe_spawn_columns`, which never returns a column any snake (player, player 2, or arena bot) occupies with its head or body, nor the column a snake is about to enter this tick, nor a column that already has a meteor in flight — the same fairness rule as Storm Mode's lightning strikes. Shower spawns additionally pass `drift = true`, which also skips any column from which a sideways-sweeping meteor (vx = ±1) would hit a snake body cell or next-head cell. Other modes keep the old fully random spawn.
- **Deflection:** eating food deflects the most imminent meteor (the lowest one in flight) with a "Deflected!" flash, giving the player a way to neutralize the closest incoming impact.
- **Impact:** a falling meteor destroys obstacles and mines in its path, despawns at the bottom edge, and kills any snake whose head or body it lands on. In Asteroids mode each crash is counted in `asteroid_impacts` and the blast carves a small crater, destroying the obstacles adjacent to the impact cell (`register_meteor_impact`).
- **Telegraphed impacts:** `meteor_impact_cells` computes where each falling meteor will crash — the first obstacle or mine on its predicted trajectory (drift-aware via `meteor_position_after`), or the bottom row. The terminal UI draws that cell as a dark-red `!` and the web UI as a dark-red cell, so the danger zone is visible before the meteor lands.
- **Autopilot:** a meteor's fall lane is a hard hazard in the A* heuristic/edge costs and in `is_safe_final_p` (a path cell in the meteor's column at or below the meteor is rejected when the meteor would reach it within the cell's true arrival step), and `path_is_safe` re-checks it per step, so the verified-safe autopilot dodges incoming meteors natively. For drifting meteors the prediction is exact: `meteor_position_after` computes where a meteor will be after `s` ticks (falling one row every 2 ticks, drifting one column per two fallen rows via bounce-aware `bounced_x`), `meteor_would_hit` answers "will this meteor occupy cell `p` within `steps` ticks" in O(1) and is a hard constraint in `is_safe_final_p`, `path_is_safe` additionally rejects paths where a drifting meteor sweeps into any cell the snake's body still occupies, and `meteor_lane_penalty`/`meteor_edge_cost` (A* and `get_boss_path` heuristics/edge costs) plus `trap_blocked_cells` penalize/block the predicted lanes for the next 10 steps.
- **Persistence:** the `meteor_showers` and `asteroid_impacts` counters are stored in `SaveState` and `HistoryState` (save/load and time-rewind stay consistent), and `meteors` were already persisted. The status bar shows `Showers: <count>` and `Impacts: <count>` counters in this mode.
- **Tests:** see [tests/test_asteroids_mode.rs](../../tests/test_asteroids_mode.rs).

## Tournament Mode

`GameMode::Tournament` (menu entry "Tournament") runs an 8-entrant single-elimination bracket (`Tournament` in `src/game/tournament.rs`, created on `reset()` via `Tournament::new`):

- **Entrants:** the player (named after `player_name`, else "You", seeded at `stats.player_elo`) plus 7 bot entrants with flavor names and ELO ratings within ±150 of the player's, all shuffled into the bracket by the game's `StdRng`.
- **Bracket:** `rounds` holds the slot levels `[8, 4, 2, 1]`; `current_match()` pairs slots `(2m, 2m+1)` of the current round. `round_label()` names the rounds Quarterfinal / Semifinal / Final. 7 matches decide the tournament.
- **Matches:** each match is a live arena duel between the player snake and `player2`, both driven by the verified-safe autopilot (`handle_autopilot_moves` treats Tournament like BotVsBot, and `hunting_enabled` allows cut-offs). When a snake dies, `handle_tournament_match_end` records the result: side A/B win advances the winner into the next round's slot, a double-kill is a draw and the same pairing is replayed.
- **ELO:** every result updates both entrants with `elo_ratings` (K=32, standard expected-score formula, clamped at 0). When the final is decided, `champion` is set, `stats.tournaments_played`/`tournaments_won` are updated (unlocking the `TournamentChampion` achievement when the player wins), and `stats.player_elo` is synced to the player entrant's final rating. Because the player's entrant is bot-driven, `reset()` sets `used_bot_this_session`, so any high score is recorded under `[BOT]`.
- **Between matches:** `start_tournament_match` respawns both snakes at the usual versus-mode spawn points, clears dynamic hazards (lasers, mines, meteors, portals, power-ups, bosses, snow, lightning, flow field), respawns food on a free cell, and announces the next pairing in the chat log, as a floating text, and in the status bar (`Trophy: <round> <m>/<n>: A (elo) vs B (elo)` via `Tournament::status_line`).
- **Persistence:** the full `tournament` state is stored in `SaveState` (save/load) and `HistoryState` (time-rewind with `Z`); loading an old Tournament save without bracket data rebuilds a fresh bracket.
- **Tests:** see [tests/test_tournament_mode.rs](../../tests/test_tournament_mode.rs).
## Tornado Mode

`GameMode::Tornado` (menu entry "Tornado Mode") is a survival mode wrapped in a permanent twister (`weather` is forced to `Weather::Tornado` every tick, and the random weather roll skips this mode):

- **Vortex:** a single tornado vortex (`tornado`, rendered as a dark-grey `@` terminal / dark-grey cell web) spawns on a random empty cell on reset (never on a snake, an obstacle, or an entity, with a `safe_zone_margin + 2` margin) and reforms the same way if it is ever lost (e.g. after a respawn).
- **Scheduled gusts:** on every 6th tick the vortex drifts one cell in a shuffled random direction and `tornado_gusts` is incremented (shown as `Gusts: <count>` in the status bar).
- **Twister spins:** after every scheduled gust, a full spin (`tornado_spin`) sweeps the board: the food, bonus food, poison food, and power-up are relocated to random empty cells (`tornado_relocate_items`) and up to 3 obstacles are lifted and re-dropped on new random empty cells (`tornado_scatter_obstacles`), so the obstacle count never changes. Spins are counted in `tornado_spins` (shown as `Spins: <count>` in the status bar).
- **Board-wide gusts:** on every 8th tick `manage_tornado_gust` fires a gust in one shared random direction that shoves every snake (player, player 2, arena bots) exactly one cell via `gust_destination`, and drags the food one cell with it. A shove uses `move_to(dest, false)`, so it never grows the snake. The gust never shoves out of bounds (unless the board wraps, in which case the destination wraps like a normal move), into an obstacle, into any snake's body, onto the vortex, or into anything lethal or occupied (mines, black hole, active lightning column, bosses, lasers, meteors, turrets); snakes are shoved sequentially so earlier shoves are respected by later checks, and a blocked destination simply fizzles. Shoved snakes get a "Gust!" floating text. The autopilot survives natively: every shove lands on a verified-safe cell and the bot re-plans its verified-safe route from the shoved position next tick.
- **Fairness:** a gust never lands on a snake's body, on the cell any snake is about to enter this tick, or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, lasers, meteors, xp gems, equipment boxes, crops, eggs, resources, the KoTH zone, flags), so the twister is dangerous but never cheap. Spin relocations/scatterings follow the same rule: nothing is ever dropped on a snake, on an entity, on a snake's next cell, or on/next to the vortex itself (`tornado_forbidden_cells`).
- **Destruction:** when the vortex moves, all obstacles and mines on and around its new cell (Manhattan distance <= 1) are sucked in and removed.
- **Item pull:** every tick the vortex drags the food, bonus food, poison food, and power-up one cell towards itself (20% chance each, reusing the black-hole pull) if the destination is free.
- **Lethality:** a snake (player, player 2, or arena bot) whose head enters the vortex cell dies, unless invincible.
- **Autopilot:** the vortex cell is a hard hazard in `is_safe_final_p` (so verified-safe path checks reject it at every step), carries a large heuristic penalty and edge cost in both `astar_search` and `get_boss_path`, and is a blocked cell in `trap_blocked_cells`, so bots and bosses route around the twister natively. Relocated/scattered obstacles remain the same hard constraints in the A* search, `path_is_safe` verification, and trap checks, and targets are recomputed every tick, so the autopilot handles the reshuffled board natively too.
- **Persistence:** `tornado`, `tornado_gusts`, and `tornado_spins` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_tornado_mode.rs](../../tests/test_tornado_mode.rs) and the save/load test in `src/game/tests.rs`.

## Earthquake Mode

`GameMode::Earthquake` (menu entry "Earthquake Mode") is a survival mode wrapped in a permanent earthquake (`weather` is forced to `Weather::Earthquake` every tick, and both the random weather re-roll and the ambient earthquake effect are disabled in this mode so only the fair scheduled tremors shape the board):

- **Scheduled tremors:** on every 5th tick a tremor fires (`manage_earthquake`). With 50% probability an existing rubble pile collapses, otherwise a new rubble tile is dropped on a random empty cell, up to a cap of 10% of the board. Rubble is stored in `rubble_tiles` and mirrored into `obstacles`, so it blocks movement for the player, the autopilot, and all arena bots like any other obstacle.
- **Fairness:** a tremor never drops rubble on a snake's body, on any entity (food, power-ups, portals, mines, bosses, turrets, companion, goblin, lava, ...), or on the cell any snake is about to move into this tick.
- **Settling:** at tick 200 of every 250-tick cycle, all accumulated rubble crumbles away ("Settling!"), keeping the board playable.
- **Shaking loose by food:** eating food removes all rubble tiles adjacent to the eaten cell ("Shook N rubble loose!").
- **Persistence:** rubble survives respawns and is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent), along with the `tremor_count` counter. If another effect clears obstacles (e.g. the Bomb power-up), stale rubble records are pruned on the next tick.
- **Autopilot:** rubble is a hard obstacle in the A* heuristic/edge costs and in `is_safe_final_p` verified-safe path checks, so the autopilot routes around the shifting rubble field natively.
- **Rendering:** rubble piles are drawn as dark-yellow `^` (terminal) / brown cells (web), and the status bar shows a `Tremors: <count>` counter.
- **Tests:** see [tests/test_earthquake_mode.rs](../../tests/test_earthquake_mode.rs).
## Meteor Shower Mode

`GameMode::MeteorShower` (menu entry "Meteor Shower Mode", index 55) is a survival mode under a constant meteor bombardment:

- **Ambient meteors:** the per-tick spawn chance is raised to 15% (the Asteroids-mode rate; 1% in other modes). A meteor enters at the top of a random column and falls one cell every 2 ticks, destroying obstacles and mines in its path and killing any snake whose head or body it touches.
- **Scheduled strikes:** on every 6th tick a strike is guaranteed at a fair column chosen by `pick_safe_strike_column` — never a column any snake currently occupies (head *or* body, since a falling meteor sweeps its whole column) or is about to enter. The chosen column is exposed as `meteor_strike_column`.
- **Clear skies:** `weather` is pinned to `Weather::Clear` every tick (and the mode is excluded from the random weather reroll), so incoming meteors always stay visible.
- **Autopilot:** falling meteors are already hard hazards in the A* heuristic/edge costs and in `is_safe_final_p`, which rejects any cell in a meteor's column that the meteor will reach within the cell's arrival step count, so the bot routes around meteor columns natively.
- **Persistence:** the `meteor_strikes` counter is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent; meteors themselves were already persisted), and the status bar shows a `Meteors: <count>` counter.
- **Tests:** see [tests/test_meteor_shower_mode.rs](../../tests/test_meteor_shower_mode.rs).
        }

## Seismic Mode

`GameMode::Seismic` (menu entry "Seismic Mode" (entry 56 in the main menu; Quake Mode is 51, Volcano Mode is 52, Tornado Mode is 53, Earthquake Mode is 54, Meteor Shower Mode is 55)) is a survival mode wrapped in a permanent earthquake (`weather` is forced to `Weather::Earthquake` every tick):

- **Scheduled tremors:** on every 6th tick a tremor is guaranteed to shake a random row (on top of the usual ambient earthquake chance). The shaken row crumbles: all obstacles in it are destroyed, bosses standing in it lose 5 health (a kill grants score, bestiary/quest/bounty progress, and a chance to drop an equipment box), and up to 5 fresh rubble piles spawn on random empty cells of that row, up to a cap of 10% of the board. Rubble tiles are stored in `seismic_rubble` and mirrored into `obstacles`, so they block movement for the player, the autopilot, and all arena bots like any other obstacle.
- **Fairness:** a scheduled tremor never shakes the row a snake currently occupies or the row its head is about to enter this tick (`pick_seismic_row`), and rubble never spawns on a snake's body, on any entity (food, power-ups, portals, mines, bosses, turrets, companion, goblin, ...), or on the cell any snake is about to move into this tick.
- **Settling:** at tick 200 of every 250-tick cycle, the ground settles ("Settling!") and all accumulated rubble crumbles away, keeping the board playable.
- **Bracing by food:** eating food clears all rubble tiles adjacent to the eaten cell ("Braced N rubble!").
- **Autopilot:** rubble lives in `obstacles`, which the A* search, `path_is_safe`/`is_safe_final_p` verified-safe checks, and trap avoidance already treat as hard constraints, so the autopilot routes around fresh rubble natively.
- **Persistence:** rubble survives respawns and is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent), along with the `seismic_quakes` counter. If another effect clears obstacles (e.g. the Bomb power-up), stale rubble records are pruned on the next tick.
- **Rendering:** rubble is drawn as dark-grey `#` (terminal) / grey cells (web) on top of the red `X` obstacles, and the status bar shows a `Quakes: <count>` counter.
- **Tests:** see [tests/test_seismic_mode.rs](../../tests/test_seismic_mode.rs).


## Battle Royale Safe Zone

`GameMode::BattleRoyale` shrinks the playable area over time: every `Game::ZONE_SHRINK_INTERVAL` (10 s) of wall-clock time `safe_zone_margin` grows by one cell on every side (capped at `(min(width, height) / 2) - 2`), and the engine treats the zone border as a wall, so a head entering it dies. Food, bonus food, power-ups, portals, meteors, lasers, and spawns all respect the current margin.

The autopilot is fully zone-aware (see the "Shrinking-zone awareness" bullet above and [tests/test_bot_zone_aware.rs](../../tests/test_bot_zone_aware.rs)):

- **Hard constraint:** `is_safe_final_p` returns `false` for any cell outside `within_safe_zone(p, steps)` — before every other hazard check and regardless of invincibility, exactly mirroring the engine's lethal wall/zone condition (`x <= margin || x >= width-1-margin || y <= margin || y >= height-1-margin`). Outside Battle Royale `within_safe_zone` never rejects (the map edge is handled by `get_final_p`), so other modes are unchanged.
- **Shrink projection:** `zone_margin_at(steps)` projects the margin `steps` ticks ahead using `last_shrink_time.elapsed()` plus `steps * Game::MAX_TICK_DURATION` (500 ms, an upper bound on any tick), capped at the maximum margin — so a path is verified against the zone it will face when each cell is actually reached, not the zone of today.
- **Search and fallbacks:** `astar_search` never expands into out-of-zone cells (even target cells), `path_is_safe` rejects paths that cross the projected border, and the trap-avoidance/space flood fills (`path_keeps_tail_reachable`, `reachable_space_after_path`) only count in-zone cells via the zone-aware `flood_fill_reachable`. The flow field uses `zone_margin_now()` plus the same `is_safe_final_p` check, so flow-field bots are covered too.
## Eclipse Mode

`GameMode::Eclipse` (menu entry "Eclipse Mode") is a survival mode wrapped in a permanent eclipse (`weather` is forced to `Weather::Eclipse` every tick, and ambient weather randomization is skipped in this mode):

- **Darkness:** rendering visibility is restricted to a radius of 6 around each snake's head (terminal and web UI both), so only the neighborhood of your snake is drawn. Collision and game logic still use the full board state, and the autopilot navigates on the full board state (as in Fog Of War mode), so the bot is unaffected by the dark.
- **Solar flares:** on every 6th tick a scheduled solar flare sets `eclipse_flare = 3`, revealing the whole board for 3 ticks ("Solar flare!"), and the flare timer counts down each tick until the board is dark again.
- **Sunlit:** eating food sets `eclipse_flare` to at least 6 ("Sunlit!"), a manual way to lift the darkness without incrementing the flare counter.
- **Persistence:** the `eclipse_flares` counter and the `eclipse_flare` timer are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows a `Flares: <count>` counter.
- **Tests:** see [tests/test_eclipse_mode.rs](../../tests/test_eclipse_mode.rs).


## Sandstorm Mode

`GameMode::Sandstorm` (menu entry "Sandstorm Mode") is a survival mode wrapped in a permanent sandstorm (`weather` is forced to `Weather::Sandstorm` every tick, and the ambient weather randomization is skipped in this mode):

- **Blowing dust:** rendering visibility is restricted to a radius of 6 around each snake's head (terminal and web UI both). Collision and game logic still use the full board state, and the autopilot navigates on the full board state (as in Fog Of War mode), so the bot is not blinded by the dust.
- **Scheduled sandblasts:** on every 6th tick a sandblast is guaranteed (on top of the usual ambient sandstorm chance) and drifts the food to a random adjacent cell ("Sandblast!"). If every adjacent cell is blocked, the food stays put but the blast still counts.
- **Sandfall:** on every 6th tick a sand drift (`%`) also falls on a random empty cell, up to a cap of 10% of the board. Sand tiles are stored in `sand_tiles` and mirrored into `obstacles`, so they block movement for the player, the autopilot, and all arena bots like any other obstacle.
- **Fairness:** neither the shifted food nor a new drift ever lands off the board, on a snake's body, on any entity (bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors, the tornado vortex, lava, rubble), on an obstacle, or on the cell any snake is about to move into this tick (`manage_sandstorm`).
- **Gale cycle:** at tick 200 of every 250-tick cycle, all accumulated sand blows away ("The sand blows away!"), keeping the board playable.
- **Clearing by food:** eating food clears all sand tiles adjacent to the eaten cell ("Cleared N sand!").
- **Autopilot:** sand is a hard obstacle in the A* heuristic/edge costs, the `is_safe_final_p` verified-safe path checks, and the trap-avoidance flood fills (all through `obstacles`), and the autopilot recomputes its target list every tick so it chases the wind-shifted food natively.
- **Persistence:** `sand_blasts`, `sand_tiles`, and `sand_drifts` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Blasts: <count>` and `Sand: <count> | Drifts: <count>` counters, and sand is rendered as a yellow `%` (terminal) / tan cell (web).
- **Tests:** see [tests/test_sandstorm_mode.rs](../../tests/test_sandstorm_mode.rs).
- **Tests:** see [tests/test_sandstorm_mode.rs](../../tests/test_sandstorm_mode.rs).

## Bomb Tag

`GameMode::BombTag` (menu entry "Bomb Tag") is an arena elimination mode: the player faces 3 arena bots (ids 3-5), and one participant is randomly chosen on reset to carry a ticking bomb:

- **Bomb state:** `bomb_carrier` (1 = player, 2 = player 2, 3+ = bot index + 3), `bomb_timer` (counts down from `Game::BOMB_TIMER_MAX` = 30 every tick), and `bomb_tags` (pass counter). The status bar shows `Bomb: <carrier> (<ticks>) | Tags: <count>`, and the carrier's head is rendered as a red `!` (terminal) / red cell (web).
- **Passing:** if the carrier's head is orthogonally adjacent (Manhattan distance 1) to another alive snake's head at the end of a tick, the bomb passes to that snake, the timer resets to `BOMB_TIMER_MAX`, `bomb_tags` is incremented, and "Tagged!" is shown.
- **Explosion:** when the timer reaches 0 (or `match_time` reaches 0), the carrier is eliminated: a bot is removed from `bots` (with particles and "Boom!"), player 2 is removed, or the player loses a life through `handle_death` (bots are preserved across player respawns in this mode). The bomb then respawns on a random other alive participant with a fresh timer.
- **Winning:** when only one participant remains (no player 2 and no bots) and the player is alive, `handle_win` declares the player the winner (Game Won / high-score flow). If the player's lives run out, the standard Game Over flow applies.
- **Autopilot:** arena bots and the player bot run the usual verified-safe pipeline; additionally, a carrier prepends up to 4 targets — the in-bounds cells adjacent to the nearest rival's head (`bomb_rival_adjacent_targets`) — so it actively chases, while non-carriers flee the carrier via the predicted-enemy-occupancy hard constraints in `path_is_safe`. Passing then happens through normal movement.
- **Persistence:** `bomb_carrier`, `bomb_timer`, and `bomb_tags` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_bomb_tag.rs](../../tests/test_bomb_tag.rs).


## Acid Rain Mode

`GameMode::AcidRain` (menu entry "Acid Rain Mode") is a survival mode wrapped in a permanent acid rain (`weather` is forced to `Weather::Rain` every tick and ambient weather re-rolls are skipped in this mode):

- **Downpours:** every 6th tick a scheduled downpour drops one acid puddle on a random fair empty cell, capped at 10% of the board. Puddles are stored in `acid_puddles` and mirrored into `obstacles`, so they are lethal on entry and block movement for the player, the autopilot, and all arena bots like any other obstacle.
- **Fairness:** acid never falls on a snake's body, on the cell any snake is about to move into this tick, on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, bosses, turrets, companion, goblin, lasers, meteors, the tornado vortex, ...), on an obstacle, or on another puddle.
- **Evaporation:** at tick 200 of every 250-tick cycle the rain breaks and all accumulated acid evaporates ("The acid evaporates!"), keeping the board playable.
- **Neutralizing by food:** eating food neutralizes all acid puddles adjacent to the eaten cell ("Neutralized N acid!").
- **Initial rainfall:** on reset the mode seeds a small initial acid rainfall (1/80 of the board) on fair cells.
- **Autopilot:** puddles are hard obstacles in the A* search and the `is_safe_final_p` verified-safe path checks, so the bot rides out the rain natively. Rain weather also raises the ambient bonus-food spawn chance (see `manage_bonus_food`).
- **Persistence:** `acid_puddles` and `acid_drops` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); stale puddle records are pruned on the next tick if another effect clears obstacles (e.g. the Bomb power-up).
- **Rendering:** acid puddles are drawn as a green `"` (terminal) / green cell (web), and the status bar shows `Acid: <count> | Drops: <count>` counters.
- **Tests:** see [tests/test_acid_rain_mode.rs](../../tests/test_acid_rain_mode.rs) plus a save/load unit test in `src/game/tests.rs` (`test_save_and_load_acid_rain_mode`).

## Monsoon Mode

`GameMode::Monsoon` (menu entry "Monsoon Mode") is a permanent rainstorm mode:

- **Weather lock:** the mode forces `Weather::Rain` on reset and on every tick; the ambient weather re-roll is skipped in this mode (rain streaks are drawn on both UIs by the existing `Weather::Rain` renderer).
- **Downpours:** on every 6th tick a scheduled downpour fires (`manage_monsoon`): a fresh puddle splashes down on a random fair empty cell via `get_random_empty_point`, and one random existing puddle swells onto an adjacent fair cell. Puddles are capped at 10% of the board.
- **Fairness:** a puddle never lands off the board, on a snake's body, on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors), on an obstacle, or on the cell any snake is about to move into this tick.
- **Puddles as obstacles:** puddle tiles are stored in `puddle_tiles` and mirrored into `obstacles`, so they block movement for the player, the autopilot, and all bots like any other obstacle (hard hazard in the A* search, `is_safe_final_p` verified-safe checks, and trap-avoidance flood fills).
- **Drain cycle:** at tick 200 of every 250-tick cycle the rain pauses and all accumulated puddles dry up ("The rain drains away!"), keeping the board playable.
- **Clearing by food:** eating food drains all puddles adjacent to the eaten cell ("Drained N puddles!").
- **Persistence:** `puddle_tiles` and `monsoon_downpours` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Puddles: <count> | Downpours: <count>`, and puddles are rendered as a blue `≈` (terminal) / blue cell (web).
- **Tests:** see [tests/test_monsoon_mode.rs](../../tests/test_monsoon_mode.rs).

## Wildfire Mode

`GameMode::Wildfire` (menu entry "Wildfire Mode", entry 61 in the main menu) is a survival mode wrapped in a permanent wildfire (`weather` is forced to `Weather::Wildfire` every tick, and the ambient weather randomization is skipped in this mode):


## Singularity Mode

`GameMode::Singularity` (menu entry "Singularity Mode", entry 79 in the main menu) is a survival mode built around a permanent black hole:

- **Permanent singularity:** in this mode `manage_black_hole` skips the random spawn/despawn rolls; `manage_singularity` (re)spawns the hole on a fair cell if it is missing — never on a snake, an entity, an obstacle, the food, or within Manhattan distance 6 of the spawn point (`get_random_empty_point` with `safe_zone_margin + 2`) — so a hole is always on the board. The hole cell is lethal on entry (existing `hit_black_hole` check), and the hole swallows adjacent obstacles and mines every tick (`singularity_consume_adjacent`).
- **Drift:** on every 8th tick the hole creeps one cell in a random direction, but only onto a fair cell (`singularity_cell_free`: in bounds within the safe-zone margin, not on any snake body or the cell any snake is about to enter, not on the hole itself, an obstacle, the food, any entity, or a hazard). Drifts are counted in `singularity_drifts` and announced with "The singularity drifts!".
- **Gravity pulse:** on every 6th tick a scheduled pulse (`singularity_pulses += 1`, "Gravity pulse!") pulls every snake (player, player 2, arena bots) one cell toward the hole along the dominant axis (`singularity_step_destination`, wrap-aware) and drags the food, bonus food, poison food, and power-up one cell along. Snakes are pulled sequentially so earlier pulls are respected by later fairness checks.
- **Fairness:** a pull only happens when the destination is fair — never out of bounds, onto the hole, an obstacle, a mine, a lightning column, a boss/laser/meteor/turret/the tornado vortex, a snake's body, or the cell any snake is about to enter — so a pulse can never directly kill a snake; the hole only kills when a snake moves into it. The same fairness gate protects the drift and the item drags.
- **Gravity shield:** eating food sets `singularity_shield = Game::SINGULARITY_SHIELD_TICKS` (6) ticks ("Shielded!"); the shield counts down every tick and while it is active the pulse skips the player ("Shielded!" instead of "Pulled!").
- **Feeding the void:** eating food also swallows the obstacles and mines adjacent to the eaten cell ("Fed the void! Cleared N!"), giving the player a way to clear debris around their meals.
- **Autopilot:** the black-hole cell is already a hard hazard in the A* edge costs and the `is_safe_final_p` verified-safe checks (as in every mode), so the bot never paths into the hole; since pulses are non-lethal by fairness (they only ever move a snake onto a fair cell), the verified path pipeline stays valid and the bot rides out the gravity natively, replanning from its actual position each tick (same as Tornado gusts).
- **Persistence:** `singularity_pulses`, `singularity_drifts`, and `singularity_shield` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent), and `black_hole` was already persisted. The status bar shows `Pulses: <count> | Drifts: <count>` plus a `Shield: <ticks>` indicator while the shield is active. In Singularity mode the hole is rendered as a dark-magenta `O` (terminal) / dark-magenta cell (web); otherwise it is the usual dark-grey `O` / dark cell.
- **Tests:** see [tests/test_singularity_mode.rs](../../tests/test_singularity_mode.rs), [tests/test_singularity_void.rs](../../tests/test_singularity_void.rs) (feeds-the-void and pulse-fairness), and the save/load test in `src/game/tests.rs`.

- **Initial ignitions:** at reset, two random fair cells ignite (`ignite_initial_wildfire`).
- **Scheduled blazes:** on every 6th tick a blaze fires: `spread_wildfire` spreads the fire from a random burning-frontier cell to up to 3 random orthogonal neighbors, and `ignite_wildfire_cell` starts one new ignition on a random fair cell. The burning field is capped at 12% of the board. New ignitions are counted in `wildfire_ignitions` and shown as `Ignitions: <count>` in the status bar.
- **Lethality:** burning cells are stored in `fire_tiles` and mirrored into `obstacles`, so entering a burning cell kills the snake, and the autopilot treats fire as a hard obstacle natively (A* heuristic/edge costs, `is_safe_final_p` verified-safe path checks, and trap-avoidance flood fills all see it through `obstacles`).
- **Burn-down:** every burning cell carries a per-cell timer (`fire_ttl`, `FIRE_TTL` = 30 ticks). Each tick the timers count down; when a cell's timer expires it stops burning, leaves `obstacles`, and smolders into walkable `ash_tiles`.
- **Fairness:** fire never ignites or spreads onto a snake's body, the cell any snake is about to enter this tick, or any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, lasers, meteors, the tornado vortex, lava, rubble, snow, sand, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources) — see `is_wildfire_spread_blocked`.
- **Douse cycle:** at tick 200 of every 250-tick cycle rain douses the whole fire ("Rain douses the wildfire!") and washes all ash away, keeping the board playable.
- **Douse by food:** eating food douses all burning cells adjacent to the eaten cell ("Doused N flames!"), which smolder into ash.
- **Persistence:** `fire_tiles`, `fire_ttl`, `ash_tiles` and `wildfire_ignitions` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). If another effect clears obstacles (e.g. the Bomb power-up), stale fire records are pruned on the next tick.
- **Rendering:** burning cells are drawn as a red `*` (terminal) / red cell (web), ash as a dark-grey `.` (terminal) / dark-grey cell (web), the wildfire weather draws dark-red ember specks (both UIs), and the status bar shows `Fire: <count> | Ignitions: <count>` plus ` | Weather: Wildfire`.
- **Tests:** see [tests/test_wildfire_mode.rs](../../tests/test_wildfire_mode.rs) and the save/load unit test `test_save_and_load_wildfire_mode` in `src/game/tests.rs`.
## Event Horizon Mode

`GameMode::EventHorizon` (menu entry "Event Horizon Mode", entry 79 in the main menu) is a survival mode built around a permanent black hole (`black_hole`, rendered as a dark `O` on terminal and web):

- **Permanence:** the event horizon is seeded on a fair cell at reset (never on a snake, an obstacle, or any entity) and never despawns — the ambient random black-hole spawn/despawn in `manage_black_hole` is disabled in this mode, and `manage_event_horizon` re-seeds the hole if it was ever lost. The skies stay `Weather::Clear` (ambient weather randomization is skipped in this mode).
- **Scheduled drifts:** every 6th tick the event horizon drifts one cell in a random direction (`event_horizon_drifts`, shown as `Drifts: <count>`). Fairness: a drift never lands off the board, on a snake's body, on the cell any snake is about to enter, on an obstacle, or on any entity (food, bonus/poison food, power-ups, portals, mines, lava, the tornado vortex, goblin, bosses, companion, turrets, XP gems, equipment boxes, crops, eggs, the KotH zone, flags, resources, lasers, meteors) — if every direction is blocked the drift fizzles and the hole stays put (`manage_event_horizon` via `event_horizon_forbidden`).
- **Gravity pull:** every tick the event horizon drags the food, bonus food, poison food, and power-up one cell towards itself (`pull_point_towards_hole`), but never onto its own lethal cell or onto a snake's body. It also swallows obstacles and mines on and around its cell (`event_horizon_consume_adjacent`).
- **Surges:** every 12th tick `manage_event_horizon_surge` fires a board-wide gravity surge (`event_horizon_surges`, shown as `Surges: <count>`) that shoves every snake (player, player 2, arena bots) one cell directly towards the event horizon via `gust_destination` (the same fairness gate as the tornado gust, now also lava-aware). A shove uses `move_to(dest, false)` so it never grows the snake; if the destination is out of bounds, an obstacle, a snake body, or anything lethal/occupied (mines, the hole itself, lightning, lava, meteors, bosses, lasers, turrets, the vortex), the surge simply fizzles for that snake ("Surge!" floating text on a successful shove). Snakes are shoved sequentially so earlier shoves are respected by later checks.
- **Recoil on eating:** eating food feeds the event horizon, which recoils one cell directly away from the eaten cell (`event_horizon_recoil`), subject to the same fairness rules as a drift ("The event horizon recoils!").
- **Lethality:** entering the hole cell kills the snake (the standard `black_hole` collision path).
- **Autopilot:** the hole is a hard hazard in the A* heuristic/edge costs and in `is_safe_final_p`; in Event Horizon mode `is_safe_final_p` additionally projects the scheduled drifts — within the next `steps` ticks the hole can reach any cell within that many drifts, so any such cell is rejected at its arrival step. Surges are safe by construction (every shove lands on a verified-safe cell) and the bot re-plans from its shoved position next tick.
- **Persistence:** `black_hole`, `event_horizon_drifts`, and `event_horizon_surges` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); the status bar shows `Drifts: <count> | Surges: <count>`.
- **Tests:** see [tests/test_event_horizon_mode.rs](../../tests/test_event_horizon_mode.rs) plus save/load and recoil unit tests in `src/game/tests.rs`.
## Infection

`GameMode::Infection` (menu entry "Infection", 65) is an arena contagion mode: the player faces 3 arena bots (ids 3-5), and one random bot is chosen on reset as Patient Zero:

- **Infection state:** `infected_ids` (list of infected participant ids; 1 = player, 2 = player 2, 3+ = bot index + 3) and `infection_spreads` (spread counter). The status bar shows `Infected: <names> | Spreads: <count>`, and every infected head is rendered as a green `Z` (terminal) / green cell (web).
- **Patient Zero:** on `reset`, exactly one random bot id is seeded into `infected_ids` (`manage_infection` reseeds the same way if the list ever ends up empty, e.g. after loading a save).
- **Spreading:** every tick (`manage_infection`, called after movement/elimination each update), if an infected participant's head is orthogonally adjacent (Manhattan distance 1) to a healthy participant's head, the healthy one becomes infected: it is added to `infected_ids`, `infection_spreads` is incremented, and "Infected!" is shown in green. Infected bots stay on the board and keep hunting; if the player (id 1) is infected, the standard `handle_death("Infected!")` flow runs (costs a life; bots are preserved across player respawns in this mode) and the player is not kept in `infected_ids` (they respawn healthy).
- **Winning:** the player wins via `handle_win` by surviving uninfected until `match_time` (1000 ticks) runs out, or by being the last healthy standing when every remaining bot is infected. If the player's lives run out, the standard Game Over flow applies.
- **Autopilot:** arena bots run the usual verified-safe pipeline; additionally, an infected bot prepends up to 4 targets — the in-bounds cells adjacent to the nearest healthy rival's head (`infection_prey_adjacent_targets`) — so it actively chases, while healthy snakes flee the infected via the predicted-enemy-occupancy hard constraints in `path_is_safe`.
- **Persistence:** `infected_ids` and `infection_spreads` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_infection.rs](../../tests/test_infection.rs).

## Inferno Mode
`GameMode::Inferno` (menu entry "Inferno Mode") is a survival mode wrapped in a permanent inferno (`weather` is forced to `Weather::Inferno` every tick, and ambient weather randomization is skipped in this mode):
- **Hotspots:** on reset, 3 burning hotspots (`inferno_hotspots`) are seeded on random fair empty cells (never on a snake, an obstacle, or the food). Hotspots are permanent fixtures: they always stay in `inferno_tiles` and `obstacles` and never get a burn timer (`spawn_inferno_hotspots`).
- **Scheduled flare-ups:** on every 6th tick a flare-up is guaranteed (`inferno_flares += 1`, `flare_inferno`): fire spreads from the current fire frontier (hotspots + burning cells) onto up to `2 + hotspots` neighboring cells, capped at 15% of the board. Every newly ignited cell gets a burn timer of `Game::INFERNO_BURN_TICKS` (36) in `inferno_burn`.
- **Burn-down:** every tick each burn timer counts down; when it reaches zero the flame cell burns out (removed from `inferno_tiles`, `inferno_burn`, and `obstacles`, and counted in `inferno_burned`). Hotspots are exempt.
- **Rain cycle:** at tick 200 of every 250-tick cycle a rainstorm douses all spread fire ("Rain douses the fire!"); only the hotspots keep burning (`manage_inferno`).
- **Firebreak by food:** eating food clears all non-hotspot fire tiles adjacent to the eaten cell ("Firebreak! Cleared N flames!").
- **Lethality and autopilot:** fire is stored in `inferno_tiles` and mirrored into `obstacles`, so it kills on entry and is a hard obstacle in the A* heuristic/edge costs, the `is_safe_final_p` verified-safe path checks, and the trap-avoidance flood fills — the autopilot routes around the blaze natively.
- **Fairness:** a flare-up never spreads fire out of bounds, on a snake (head or body of any participant), on the cell any snake is about to enter this tick, or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors, the tornado vortex) or obstacle (`is_inferno_spread_blocked`).
- **Persistence:** `inferno_tiles`, `inferno_hotspots`, `inferno_burn` (via `map_as_pairs`), `inferno_flares`, and `inferno_burned` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Fire: <count> | Flares: <count>`, and flames are rendered as a dark-yellow `&` with hotspots overdrawn in red `&` (terminal) / orange and red cells (web).
- **Tests:** see [tests/test_inferno_mode.rs](../../tests/test_inferno_mode.rs).


## Minefield Mode

`GameMode::Minefield` (menu entry "Minefield Mode") is a survival mode built around a **buried minefield** (`weather` is forced to `Weather::Clear` every tick and ambient weather re-rolls are skipped, so the mines stay visible on both UIs — they were already rendered as `M` terminal / dark cell web and persisted):

- **Initial mines:** on reset a few mines (1/80 of the board) are buried on fair empty cells (never on the snake, on the food, or on an obstacle). The ambient `manage_mines` random spawn is disabled in this mode, so mines only appear via the scheduled drops.
- **Scheduled mine drops:** on every 6th tick a drop is guaranteed (`mine_drops` counter, shown as `Drops: <count>`): one new mine is buried on a random fair empty cell, capped at 10% of the board. Mines remain lethal on entry (`hit_mine` checks) and are hard hazards in the A* search and the `is_safe_final_p` verified-safe path checks, so the autopilot weaves through the field natively.
- **Fairness:** a mine never drops on a snake's body, on the cell any snake is about to move into this tick, on any entity (food, bonus/poison food, power-ups, portals, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors, the tornado vortex, the lightning column, or any other hazard tile — lava, fire, ash, rubble, snow, sand, acid, puddles), on another mine, or on an obstacle.
- **Sweep cycle:** at tick 200 of every 250-tick cycle a mine sweep clears the whole field ("The minefield is swept!").
- **Defusing by food:** eating food defuses all mines adjacent to the eaten cell ("Defused N mines!").
- **Persistence:** `mines` (already) and `mine_drops` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Mines: <count> | Drops: <count>`.
- **Tests:** see [tests/test_minefield_mode.rs](../../tests/test_minefield_mode.rs) plus a save/load unit test in `src/game/tests.rs`.


## Laser Grid Mode

`GameMode::LaserGrid` (menu entry "Laser Grid") is a solo hazard mode with scheduled, telegraphed sweeping laser beams:

- **Permanence:** weather is pinned to `Weather::Clear` every tick (ambient weather randomization is skipped in this mode) so beams always stay visible, and reset clears all lasers and the telegraph.
- **Telegraph:** on every 5th tick (`tick_counter % 6 == 5`) `manage_laser_grid` picks a beam line via `pick_safe_beam_line` — a random row (`(true, y)`) or column (`(false, x)`) that no snake occupies (head, body, or the cell any snake is about to enter, see `beam_forbidden_lines`) — and stores it in `laser_telegraph`. The line is rendered one tick in advance as a yellow `-` / `|` row/column (terminal) or yellow cells (web).
- **Firing:** on every 6th tick the telegraphed line is re-checked with `beam_line_fair` (snakes move between telegraph and fire); if the line is still fair and fewer than `Game::LASER_GRID_MAX_BEAMS` (3) hostile lasers are active, a hostile `Laser` (`player = 2`) spawns at a random edge of the line with a direction across it and sweeps the whole line (moving 2 cells per tick, destroyed at the far wall), lethal on contact through the standard `hit_laser1` flow. `laser_volleys` is incremented (status bar `Volleys: <count>`). If the line became occupied, the beam fizzles without firing — never cheap.
- **Cycle:** at tick 200 of every 250-tick cycle the grid recalibrates: all hostile beams are destroyed and the telegraph is cleared ("The grid recalibrates!"). Eating food zaps the grid, destroying all active beams (`zap_laser_grid`, "Zapped N beam(s)!").
- **Autopilot:** moving lasers and their predicted 2-cells-per-tick trajectories are already hard hazards in the A* search and the `is_safe_final_p` verified-safe checks, so the bot dodges sweeping beams natively.
- **Persistence:** `laser_volleys` and `laser_telegraph` (plus the `lasers` themselves) are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_laser_grid_mode.rs](../../tests/test_laser_grid_mode.rs) plus a save/load unit test in `src/game/tests.rs`.
## Tsunami

`GameMode::Tsunami` (menu entry "Tsunami Mode", index 69) is a survival mode wrapped in permanent torrential rain (`weather` is forced to `Weather::Rain` every tick, and the ambient weather randomization is skipped in this mode):

- **Scheduled waves:** on every `Game::TSUNAMI_PERIOD` (40th) tick, when no wave is active or warning, a wave is announced from a random edge (`tsunami_edge` 0=top, 1=bottom, 2=left, 3=right) with `tsunami_warning = Game::TSUNAMI_WARNING` (8) ticks of warning ("Tsunami from the <edge>!"). The warning edge is rendered as a cyan `~` line (terminal) / cyan cells (web), and the status bar shows `Tsunami in <n>!`.
- **Fairness (depth cap):** at announcement, `tsunami_depth` is set to the minimum line occupied by any snake's body or next-head cell, minus one. The wave therefore never sweeps a line any snake currently occupies or is about to enter — the same "never cheap" rule as Storm Mode's lightning columns and Seismic's shaken rows. Lines beyond `tsunami_depth` are permanently safe.
- **The sweep:** once the warning runs out, `tsunami_front` starts at 0 and the wave sweeps one line per tick (`sweep_tsunami_line`), from the announced edge up to `tsunami_depth`; when the front passes the depth the wave ends and `tsunami_waves` is incremented. The front line is rendered as a blue `~` line (terminal) / blue cells (web), and the status bar shows `Wave: <line> | Waves: <count>`.
- **Sweep effects:** the swept line smashes obstacles, mines, and hazard tiles (snow/sand/lava/seismic rubble/rubble) on it; loose items on it (bonus food, poison food, power-up) are washed away, and the food is relocated to a random fair empty cell (never on a snake, an entity, an obstacle, or the cell any snake is about to enter). A snake whose head is on the swept line dies (head-only kill, like Storm's lightning column; invincibility protects) — the player loses a life via `handle_death("Swept away by the tsunami!")`, player 2 is removed, bots are removed with particles.
- **Autopilot:** `tsunami_threatens` (used by `is_safe_final_p`) rejects any cell within the wave's depth whose line is at or ahead of the wave front (not yet swept), so verified-safe paths never enter the incoming wave; lines beyond `tsunami_depth` and lines already behind the front remain safe. The bot therefore rides out every telegraphed wave natively.
- **Persistence:** `tsunami_edge`, `tsunami_front`, `tsunami_depth`, `tsunami_warning`, and `tsunami_waves` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_tsunami_mode.rs](../../tests/test_tsunami_mode.rs) and the save/load unit test in `src/game/tests.rs`.

## Warp Mode

`GameMode::Warp` (menu entry "Warp Mode") is a dedicated portal mode built on the existing `portals` entity:

- **Persistence:** the ambient random portal spawn in `manage_portals` is disabled in this mode; `manage_warp` keeps a permanent pair alive — on reset (and after any respawn) it is seeded via `pick_warp_pair(8)`: each cell at least 8 (Manhattan) from every snake's head and the cell it is about to enter, never on an obstacle, item, mine, the black hole, the tornado, a snake body, the current portals, or a pending target, and the two cells at least 6 apart. The pair never despawns.
- **Teleport:** moving into one portal cell teleports the head to the other portal cell (existing engine behavior); `get_final_p` and the A* `travel_dist` already resolve portals, so the autopilot uses the warp natively and the verified-safe checks evaluate the teleport destination.
- **Scheduled relocations:** every `Game::WARP_RELOCATE_INTERVAL` (45) ticks a relocation is telegraphed: `pick_warp_pair(6)` chooses a fair target pair (>= 6 from every snake head/next cell), stored in `warp_targets` with `warp_relocate_countdown = Game::WARP_TELEGRAPH` (6). The targets are rendered as blue/dark-magenta `o` (terminal) / blue/dark cells (web) and the status bar shows `Relocate in: <ticks>`. When the countdown reaches 0, `portals` moves to the target pair ("The portals shift!") and `warp_relocations` is incremented.
- **Persistence:** `portals` (already persisted) plus `warp_targets`, `warp_relocate_countdown`, and `warp_relocations` are stored in `SaveState` and `HistoryState`, so save/load and time-rewind stay consistent. The status bar shows `Warp: Relocations: <count>`.
- **Tests:** see [tests/test_warp_mode.rs](../../tests/test_warp_mode.rs).
## Gravity Well Mode

`GameMode::GravityWell` (menu entry "Gravity Well") is a survival mode built around a permanent roaming black hole:

- **Permanent hole:** `black_hole` is seeded on a fair empty cell at reset (never on a snake, an entity, or an obstacle, with a `safe_zone_margin + 2` margin). The ambient random spawn/despawn in `manage_black_hole` is disabled in this mode (a dedicated `GameMode::GravityWell` branch there only devours adjacent obstacles/mines and pulls items at full strength), and `manage_gravity_well` re-forms the hole on a fair cell ("The gravity well reforms!") if it was ever lost. The weather is pinned to `Weather::Clear` every tick (ambient weather randomization is skipped in this mode) so the hole is always visible.
- **Pull:** every tick the hole tugs the food, bonus food, poison food, and power-up one cell towards itself (`pull_point_towards_black_hole` at full strength in this mode). Anything pulled onto the hole cell is devoured: the food respawns on a fair empty cell ("Devoured!"), the other items are swallowed.
- **Scheduled surges:** every 6th tick a surge (`gravity_well_surges`, status bar `Well Surges: <count>`) lurches the hole one cell in a randomly chosen fair direction. A surge never lands on a snake's body, on any entity (food, bonus/poison food, power-ups, portals, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors, the tornado vortex), or on the cell any snake is about to enter this tick; if every direction is blocked the surge fizzles. The hole devours the obstacles and mines it lands on.
- **Lethality:** a snake whose head enters the hole cell dies (the existing `black_hole` collision checks).
- **Autopilot:** the hole cell is already a hard hazard in `is_safe_final_p` and the A*/boss-path edge costs, so the bot routes around it natively; the hole moves at most one cell per 6 ticks and never onto a snake's next cell, so per-tick replanning stays safe.
- **Rendering:** the hole is drawn as a dark-grey `O` (terminal) / dark cell (web); surges spawn "Surge!" floating text and particles.
- **Persistence:** `black_hole` and `gravity_well_surges` are stored in `SaveState` and `HistoryState` (save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_gravity_well_mode.rs](../../tests/test_gravity_well_mode.rs).

## Conveyor Mode

`GameMode::Conveyor` (menu entry "Conveyor Mode", 73) is a survival mode played on a factory floor: the weather is pinned to `Weather::Clear` (ambient weather re-rolls are skipped in this mode) and `belt_tiles: HashMap<Point, Direction>` is seeded at reset with `max(board/40, 4)` directional belts on fair empty cells (never on a snake, an entity, an obstacle, or another belt):

- **Shove:** when a snake's head (player, Player 2, or any arena bot) enters a belt cell, `belt_shove` (applied in `calculate_final_heads` and in the bot-head loop, right after portal/wrap resolution) shoves the head one extra cell in the belt's direction. Fairness: the destination must be in bounds (or wrapped when `wrap_mode`), and free of obstacles, all snake bodies, portals, mines, bosses, turrets, lasers, meteors, the black hole, the tornado vortex, the active lightning column, wildfire tiles, and the Battle Royale safe zone — otherwise the shove fizzles and the snake stays on the belt cell. A shove never grows the snake, and a belt never shoves onto another belt's cell twice in one tick (single-step semantics).
- **Scheduled shuffle:** every 6th tick `manage_conveyor` rotates every belt 90° clockwise (`rotate_belt`), counted in `belt_shifts` (status bar `Belts: <count> | Shifts: <count>`), with a "The belts shift!" floating text. Belts that end up under a fresh obstacle are scrapped.
- **Greasing by food:** eating food rotates one random belt 90° ("Greased the belts!").
- **Autopilot:** belts are not obstacles, so the A* search and `is_safe_final_p` are unaffected; the autopilot survives natively because every shove lands on a verified-safe cell and the bot re-plans from its actual head each tick.
- **Persistence:** `belt_tiles` (serialized as `[key, value]` pairs via `map_as_pairs`) and `belt_shifts` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Rendering:** belts are drawn as yellow `^`/`v`/`<`/`>` glyphs (terminal) / orange cells (web).
- **Tests:** see [tests/test_conveyor_mode.rs](../../tests/test_conveyor_mode.rs) plus a save/load unit test in `src/game/tests.rs` (`test_save_and_load_conveyor_mode`).

## Ice Rink

`GameMode::IceRink` (menu entry "Ice Rink") is a movement-physics mode: the board is seeded with slippery **ice patches** (`ice_tiles`, ~10% of the board) and while a snake's head is on an ice patch it **cannot turn**.

- **Board setup:** on reset the mode generates the usual obstacle set plus `width * height / 10` ice patches on fair empty cells (never off the board, on a snake's body, on an obstacle, or on the food). Weather is pinned to `Weather::Clear` (ambient re-rolls are skipped in this mode).
- **Engine rule:** the rule is enforced at the single point where every snake's queued direction is applied each tick (player, player 2, and every bot): `ice_blocks_turn` ignores a queued direction that differs from the current one while the head is on an ice patch, and counts the blocked turn in `ice_slips`. Continuing straight across ice is always allowed; turning off ice works normally.
- **Autopilot:** `astar_search` never expands a turn out of an ice cell (both the initial expansion from `start` and the per-node expansion, using the incoming direction from `came_from`), `path_is_safe` rejects any path that turns on an ice cell (comparing each step's direction with the previous step's, using the snake's current direction at the first step), and `flood_fill_fallback` only continues straight when starting on ice — so the bot routes through normal floor whenever it needs to turn.
- **Persistence:** `ice_tiles` and `ice_slips` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). Ice patches render as a cyan `.` (terminal) / light-blue cell (web), and the status bar shows `Ice: <count> | Slips: <count>`.
- **Tests:** see [tests/test_ice_rink.rs](../../tests/test_ice_rink.rs) plus a save/load unit test in `src/game/tests.rs` (`test_save_and_load_ice_rink_mode`).
## Black Hole Mode

`GameMode::BlackHole` (menu entry "Black Hole") turns the ambient black hole into a permanent, roaming singularity:

- **Spawn:** on `reset()` (and re-guaranteed in `respawn()` if lost), `spawn_black_hole_fair` places the singularity on a random fair empty cell (never on an obstacle, an entity, a snake body, the tornado, or within the safe-zone margin + 2). In this mode the random spawn/despawn logic in `manage_black_hole` is bypassed — the singularity never collapses.
- **Wander:** every 6th tick (`tick_counter.is_multiple_of(6)`) the singularity drifts one cell in a shuffled random direction (`manage_singularity`). Fairness: the destination is never out of bounds, never on a snake body or the cell any snake is about to enter, and never on any entity (food, bonus/poison food, power-ups, portals, mines, the tornado vortex, goblin, bosses, companion, turrets, gems, crops, eggs, the KoTH zone, flags, resources, lasers, meteors). Wanders are counted in `singularity_wanders` (status bar: `Wanders: <count>`).
- **Pull & devour:** every tick the singularity drags the food, bonus food, poison food, and power-up one cell towards itself (`pull_point_towards_black_hole`), and devours the obstacles and mines on and around its cell (Manhattan distance <= 1).
- **Feeding frenzy:** eating food in this mode devours all obstacles and mines within Manhattan distance <= 2 of the singularity ("Devoured N!"), counted in `singularity_feeds` (status bar: `Feeds: <count>`).
- **Lethality:** a snake whose head enters the singularity cell dies (existing black-hole collision handling); the cell is rendered as a dark-grey `O` (terminal) / near-black cell (web) on both UIs.
- **Autopilot:** the black hole cell is a hard hazard in the A* edge costs/heuristics and in the `is_safe_final_p` verified-safe path checks, so the bot routes around the roaming singularity natively; wander fairness guarantees it never lands on the cell the bot is about to enter.
- **Weather:** ambient weather randomization is skipped in this mode; the sky stays clear so the singularity is always visible.
- **Persistence:** `black_hole`, `singularity_wanders`, and `singularity_feeds` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_black_hole_mode.rs](../../tests/test_black_hole_mode.rs) and the save/load unit test in `src/game/tests.rs`.

## Ricochet Mode

`GameMode::Ricochet` (menu entry "Ricochet Mode", entry 75 in the main menu) is a movement mode: the board walls bounce the snake Pong-style instead of killing it.

- **Bounce:** in `update_tick`, when `calculate_final_heads` reports a wall hit (`hit_wall1`) for the player in this mode, `ricochet_bounce` reflects the direction on the offending axis (left wall → `Right`, right wall → `Left`, top wall → `Down`, bottom wall → `Up`) and re-targets from the current head, so the bounced cell is always back inside the board. The snake's direction is set (and re-queued) to the reflected direction, `ricochet_bounces` is incremented, and "Bounce!" is shown; the wall never sets `p1_dead` in this mode.
- **Everything else stays lethal:** obstacles, mines, hazards, and self-collision (a bounce straight into your own body still kills) are unchanged, and in every other mode walls still kill via the standard `hit_wall` flow.
- **Autopilot:** `get_final_p`/`is_safe_final_p` still treat out-of-bounds cells as unsafe, so the bot keeps walls as hard constraints and never plans a bounce — but if it is ever forced into a wall it now bounces instead of dying.
- **Persistence:** `ricochet_bounces` is stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Bounces: <count>`.
- **Tests:** see [tests/test_ricochet_mode.rs](../../tests/test_ricochet_mode.rs).
## Turret Battery Mode

`GameMode::TurretBattery` (menu entry "Turret Battery Mode", entry 79 in the main menu) is a survival mode with a permanent enemy turret battery:

- **Battery:** on reset `TURRET_BATTERY_COUNT` (3) enemy turrets (`is_enemy: true`, drawn as `T`) are seeded on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, a laser, or the cell any snake is about to enter. `manage_turret_battery` re-seeds missing enemy turrets every tick, so the battery always stays armed. The weather is pinned to `Weather::Clear` on reset and every tick (the ambient weather re-roll is skipped in this mode), so every turret and beam is visible.
- **Scheduled volley:** every 6th tick a volley fires (`turret_volleys += 1`, "Volley! (N beams)"): each enemy turret fires one laser (`Laser { position: turret, direction, player: 3 }`) in a shuffled random direction. Fairness: a beam fizzles when its first cell (`turret + direction`) is out of bounds or fails `singularity_cell_free` — never onto a snake's body, the cell any snake is about to enter, an obstacle, a mine, or any entity — so a volley is dangerous but never cheap.
- **Lethality & autopilot:** enemy beams (`player != 1`) kill on contact via the existing `hit_laser` check, and laser cells/lanes are already hard hazards/penalties in the A* edge costs and the `is_safe_final_p` verified-safe checks, so the bot dodges the fire natively. Turrets also keep their ambient aimed fire (every 10 ticks toward a snake sharing a row/column).
- **Jam:** eating food sets `turret_jammed = TURRET_JAM_TICKS` (12) ("Jammed!"); while the timer is positive it counts down once per tick and scheduled volleys are suppressed.
- **Status bar:** `Turrets: <count> | Volleys: <count>` plus `Jammed: <ticks>` while jammed.
- **Persistence:** `turret_volleys` and `turret_jammed` are stored in `SaveState` and `HistoryState` (`turrets` and `lasers` were already persisted), so save/load and time-rewind stay consistent.
- **Tests:** see [tests/test_turret_battery_mode.rs](../../tests/test_turret_battery_mode.rs) and the save/load test in `src/game/tests.rs`.

## Firestorm Mode

`GameMode::Firestorm` (menu entry "Firestorm Mode") is a survival mode wrapped in a permanent firestorm (`weather` is forced to `Weather::Firestorm` every tick, and the ambient weather randomization is skipped in this mode):

- **Blaze sources:** two blaze sources (`blaze_sources`) are seeded on random empty cells at reset and burn permanently (rendered as a dark-red `B` terminal / dark-red cell web).
- **Scheduled blazes:** on every 6th tick a blaze is guaranteed and spreads flames from the burning frontier onto neighboring cells, up to a cap of 15% of the board and counted in `firestorm_blazes`. Unlike Volcano lava, fire chars through obstacles — the charred cell becomes burning ground.
- **Burn-out:** each blaze is stored in `blaze_tiles` as `cell -> remaining burn ticks` and burns out after `Game::BLAZE_BURN_TICKS` (12) ticks, leaving the charred cell open (removed from both `blaze_tiles` and `obstacles`).
- **Fairness:** a blaze never spreads off the board, onto a snake's body, onto any entity (bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KotH zone, flags, resources, lasers, meteors, lava, wildfire, acid puddles, monsoon puddles, inferno tiles, rubble, snow, sand), or onto the cell any snake is about to move into this tick (`is_firestorm_spread_blocked`).
- **Die-down cycle:** at tick 200 of every 250-tick cycle the whole firestorm dies down ("The firestorm dies down!"), keeping only the blaze sources.
- **Dousing by food:** eating food douses all blazes adjacent to the eaten cell ("Doused N blazes!").
- **Autopilot:** blazes are mirrored into `obstacles`, so they are a hard obstacle in the A* heuristic/edge costs, the `is_safe_final_p` verified-safe path checks, and the trap-avoidance flood fills, and the bot routes around the flames natively. An end-of-tick self-heal drops stale blaze records if something else (e.g. a meteor crash or a Bomb) cleared obstacles mid-tick.
- **Persistence:** `blaze_tiles` (serialized through `map_as_pairs` as `[point, ticks]` pairs), `blaze_sources`, and `firestorm_blazes` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Blaze: <count> | Blazes: <count>`, and blazes are rendered as a red `*` (terminal) / orange cell (web).
- **Tests:** see [tests/test_firestorm_mode.rs](../../tests/test_firestorm_mode.rs).

## Fortress Mode

`GameMode::Fortress` (menu entry "Fortress Mode") is a survival mode where the arena walls breathe inward:

- **Ring closure:** on every 6th tick `manage_fortress` erects the next ring of fortress walls (`fortress_walls`, mirrored into `obstacles`) on every fair cell whose ring distance (`min(x, y, width-1-x, height-1-y)`) equals the new depth. Rings are counted in `fortress_rings` and shown as `Walls: <count> | Rings: <count>` in the status bar, with a "The walls close in! (+N)" floating text. Walls render as dark-grey `#` (terminal) / grey cells (web).
- **Retraction:** at tick 200 of every 250-tick cycle all fortress walls are removed from `obstacles` and the depth resets to 0 ("The walls retract!").
- **Depth cap:** the depth is capped at `min(width, height) / 4`, so the inner keep always stays playable.
- **Fairness:** a ring cell is never placed on a snake's body, on the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, and all hazard tile sets). Additionally, a ring is not erected at all while any snake still has a body cell outside it (`ring_dist(p) < depth`), so the closing walls can never seal a snake in the outer band.
- **Autopilot:** walls are ordinary obstacles, so they are hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance — the bot rides out the fortress natively.
- **Breaching:** eating food removes the fortress walls adjacent to the eaten cell ("Breached N wall!").
- **Persistence:** `fortress_walls`, `fortress_depth`, and `fortress_rings` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); stale wall records are pruned each tick if another effect cleared the obstacles.
- **Tests:** see [tests/test_fortress_mode.rs](../../tests/test_fortress_mode.rs).

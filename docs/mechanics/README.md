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
- **Frontier targeting:** before pathfinding, each bot prepends up to 3 targets from `turf_war_frontier_targets(start, bot_id, 3)` — the nearest in-bounds cells it has not painted itself yet (its current cell, own-painted cells, obstacles, and snake bodies are excluded). Enemy-owned tiles count as frontier, so bots actively contest and reclaim rival territory. Food, bonus food, power-ups, and the goblin remain in the sinkholeget list after the frontier cells.
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
- **Scheduled relocations:** every `Game::WARP_RELOCATE_INTERVAL` (45) ticks a relocation is telegraphed: `pick_warp_pair(6)` chooses a fair target pair (>= 6 from every snake head/next cell), stored in `warp_targets` with `warp_relocate_countdown = Game::WARP_TELEGRAPH` (6). The targets are rendered as blue/dark-magenta `o` (terminal) / blue/dark cells (web) and the status bar shows `Relocate in: <ticks>`. When the countdown reaches 0, `portals` moves to the sinkholeget pair ("The portals shift!") and `warp_relocations` is incremented.
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

## Geyser Mode

`GameMode::Geyser` (menu entry "Geyser Mode") is a survival mode where two permanent geyser vents erupt on a schedule:

- **Vents:** at reset `spawn_geyser_vents` seeds 2 vents (`geyser_vents`) on fair empty cells (never on a snake, the food, another entity, or an obstacle). Vents are permanent fixtures: they stay in `geyser_jets` with `u32::MAX` ticks and in `obstacles`, and render as a dark-cyan `G` (terminal) / dark-cyan cell (web).
- **Scheduled eruptions:** on every 6th tick `manage_geyser` fires one eruption (counted in `geyser_eruptions`) from the vents in round-robin order. `erupt_geyser` sprays a steam jet up to `GEYSER_JET_RANGE` (4) cells in each cardinal direction, stopping at the board edge or any blocked cell. Jets are stored in `geyser_jets` (cell → remaining ticks) and mirrored into `obstacles`, capped at 15% of the board, and shown as `Jets: <count> | Eruptions: <count>` in the status bar, with a "Geyser erupts! (+N)" floating text. Jets render as a cyan `w` (terminal) / cyan cell (web).
- **Fairness:** `is_geyser_jet_blocked` stops a jet before the board edge, obstacles, vents, existing jets, any snake's body, the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or any entity (food, bonus/poison food, power-ups, portals, mines, black hole, tornado, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, and all hazard tile sets).
- **Dissipation:** each jet burns down one tick at a time and dissipates after `GEYSER_JET_TICKS` (6) ticks, leaving the cell open again.
- **Rest cycle:** at tick 200 of every 250-tick cycle all non-vent jets dissipate ("The geysers rest!"), while the vents remain.
- **Autopilot:** jets are ordinary obstacles, so they are hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance — the bot rides out the eruptions natively.
- **Dousing:** eating food removes the jets adjacent to the eaten cell ("Doused N jet!").
- **Persistence:** `geyser_vents`, `geyser_jets` (serialized through `map_as_pairs` as `[point, ticks]` pairs), and `geyser_eruptions` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); stale jet records are pruned each tick if another effect cleared the obstacles.
- **Tests:** see [tests/test_geyser_mode.rs](../../tests/test_geyser_mode.rs).

## Thornfield Mode

`GameMode::Thornfield` (menu entry "Thornfield Mode", index 80 in the main menu) is a survival mode built around a growing **thorn field** (`thorn_tiles`, rendered as a dark-green `Y` terminal / forest-green cell web):

- **Initial briars:** on reset, `max(board/100, 2)` thorns are seeded on fair cells (never on a snake, an entity, an obstacle, or the cell any snake is about to enter) via `spawn_initial_thorns`. The weather is pinned to `Weather::Clear` every tick (ambient weather randomization is skipped in this mode) so the briars always stay visible.
- **Scheduled blooms:** on every 6th tick `manage_thornfield` fires a bloom (`thorn_blooms += 1`, shown as `Blooms: <count>`): up to 3 thorns spread from the current thorn frontier onto random orthogonal neighbors, capped at `board/12` (~8% of the board). A bloom announces itself with "The briars bloom! (+N)".
- **Double lethality:** thorns are mirrored into `obstacles`, so entering a thorn cell kills the snake like any wall — and each thorn also **lashes** at every orthogonally adjacent cell: a head ending its tick within Manhattan distance 1 of a thorn dies (`thorn_lashes`, checked for the player, player 2, and every arena bot in `update_tick`, gated by invincibility just like mines).
- **Fairness:** a bloom never places a thorn out of bounds, on an obstacle, on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, tornado, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, KoTH zone, flags, resources, lasers, meteors, belts, ice, and every other hazard tile set), or on any cell whose **lash zone** (itself plus its 4 neighbors) contains a snake body cell or the cell any snake is about to enter this tick (`is_thorn_spread_blocked`) — so a bloom can never cheap-kill.
- **Wither cycle:** at tick 200 of every 250-tick cycle all thorns wither away ("The thorns wither!"), keeping the board playable.
- **Pruning by food:** eating food prunes all thorns adjacent to the eaten cell ("Pruned N thorns!").
- **Autopilot:** thorn cells are ordinary obstacles, and their whole lash zone is an additional hard hazard in `is_safe_final_p` (verified-safe path checks), penalized in the `astar_search` heuristic, and blocked in `trap_blocked_cells` flood fills — so the bot routes around the briars natively and never gets pricked.
- **Persistence:** `thorn_tiles` and `thorn_blooms` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); stale thorn records are pruned each tick if another effect cleared the obstacles.
- **Tests:** see [tests/test_thornfield_mode.rs](../../tests/test_thornfield_mode.rs) plus the save/load unit test `test_save_and_load_thornfield_mode` in `src/game/tests.rs`.

## Mirage Mode

`GameMode::Mirage` (menu entry "Mirage Mode") is a deception mode: illusory food mirages try to lure the snake into wasted trips.

- **Scheduled spawns:** on every 6th tick `manage_mirage` calls `spawn_mirage`, which places one mirage (`mirage_foods`, cell → remaining ticks, capped at 10% of the board, counted in `mirage_spawns`) on a random fair cell, with an "A mirage appears!" floating text. Mirages render as a magenta `?` (terminal) / orchid cell (web) and are shown as `Mirage: <count> | Spawns: <count>` in the status bar.
- **Fairness:** `mirage_spawn_blocked` rejects any cell on a snake's body, on the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), on an obstacle, on another mirage, or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, tornado, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, and the belt/ice/geyser/thorn tiles).
- **Shattering:** a mirage is not food — when any snake's head walks onto its cell, the mirage shatters ("Mirage shattered! (-N)") with no growth and no score.
- **Fading:** each mirage burns down one tick at a time and vanishes after `MIRAGE_TICKS` (30) ticks; at tick 200 of every 250-tick cycle all mirages clear ("The heat shimmers away!").
- **Dispelling:** eating real food clears every mirage on the board ("Dispelled N mirage(s)!").
- **Autopilot:** mirages live in their own `mirage_foods` map and are never added to the bot's target list, so the autopilot only chases real food and is never lured.
- **Persistence:** `mirage_foods` (serialized through `map_as_pairs` as `[point, ticks]` pairs) and `mirage_spawns` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_mirage_mode.rs](../../tests/test_mirage_mode.rs).
## Avalanche Mode

`GameMode::Avalanche` (menu entry "Avalanche Mode") is a survival mode where boulders roll across the board in straight rows:

- **Scheduled rockfalls:** on every 6th tick `manage_avalanche` fires a rockfall (counted in `avalanche_rolls`) as long as fewer than `AVALANCHE_MAX_BOULDERS` (6) boulders are rolling. `spawn_boulder` picks a random row and side edge (left spawns moving `Right`, right spawns moving `Left`) and pushes a `Boulder { position, dir }` into `boulders`, with a "Rockfall!" floating text.
- **Rolling:** every tick `roll_boulders` moves each boulder one cell in its direction. A boulder crushes obstacles and hazard tiles in its path (`remove_obstacle_and_hazards`) and crumbles when it rolls off the far edge.
- **Lethality:** a boulder landing on a snake's head or body kills it (`hit_boulder1`/`hit_boulder2` in the player death checks, plus a `boulders` check in the bot `bots_to_remove` loop).
- **Fairness:** a boulder never breaks loose in a row any snake occupies (head or body) or is about to enter this tick (player/p2 use their current direction, bots their queued direction), and never on an entity (food, power-ups, portals, mines, black hole, tornado, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, other boulders).
- **Settling:** at tick 200 of every 250-tick cycle all boulders crumble ("The avalanche settles!").
- **Blasting:** eating food removes the boulders adjacent to the eaten cell ("Blasted N boulder!").
- **Autopilot:** the bot predicts each boulder's exact lane: `boulder_position_after` projects a boulder `steps` ticks ahead, `boulder_would_hit` reports whether a boulder occupies a cell at a given step. `is_safe_final_p` rejects any cell a boulder will occupy at its arrival step; `path_is_safe` rejects paths whose cells (or still-occupied body cells) a boulder sweeps within the path horizon; the A* expansion's blocked set includes every predicted boulder lane; and `boulder_edge_cost`/`boulder_lane_penalty` add large costs near rolling lanes, so the bot dodges the rockfall natively.
- **Weather:** the skies are pinned to `Weather::Clear` (ambient weather randomization is skipped in this mode) so boulders always stay visible.
- **Rendering/status:** boulders render as dark-yellow `O` (terminal) / goldenrod cells (web), and the status bar shows `Boulders: <count> | Rolls: <count>`.
- **Persistence:** `boulders` and `avalanche_rolls` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_avalanche_mode.rs](../../tests/test_avalanche_mode.rs) plus a save/load unit test in `src/game/tests.rs`.

## Pinball Mode

`GameMode::Pinball` (menu entry "Pinball Mode") is a movement mode where bumpers on the board deflect the snake like pinball flippers instead of killing it:

- **Bumpers:** at reset `spawn_pinball_bumpers` seeds bumpers (`pinball_bumpers`, mirrored into `obstacles`) on fair cells, up to a cap of `clamp(min(width, height) / 5, 4, 8)`. Bumpers render as a magenta `O` (terminal) / magenta cells (web), and the status bar shows `Bumpers: <count> | Bumps: <count>`.
- **Deflection:** when the player's final head would enter a bumper cell, `pinball_deflect` picks a free perpendicular lane (up/down for horizontal approaches, left/right for vertical ones), falling back to a straight bounce-back; the reflected direction is re-queued, `pinball_bumps` is incremented, +5 score is awarded ("Bump! +5"), and the game beeps. If every deflection lane is blocked (obstacle, snake body, out of bounds), the bumper stays lethal and the normal obstacle-death flow applies.
- **Shuffles:** on every 6th tick `manage_pinball` either tops up towards the bumper cap or relocates one existing bumper to a fresh fair cell (`pinball_relocations`, "Bumper moved!"); at tick 200 of every 250-tick cycle all bumpers are removed from `pinball_bumpers` and `obstacles` ("The bumpers reset!"). The skies are pinned to `Weather::Clear` every tick so the bumpers always stay visible.
- **Fairness:** a bumper is never placed on a snake's body, on the cell any snake is about to enter this tick (`pinball_next_cells`: the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or on any entity or hazard tile (`pinball_avoid`: food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, and all hazard tile sets). Stale bumper records are pruned each tick if another effect cleared the obstacles.
- **Autopilot:** bumpers are ordinary obstacles, so they are hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance — the bot routes around them natively and never needs to bounce.
- **Clearing:** eating food removes the bumpers adjacent to the eaten cell ("Cleared N bumper(s)!").
- **Persistence:** `pinball_bumpers`, `pinball_bumps`, and `pinball_relocations` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_pinball_mode.rs](../../tests/test_pinball_mode.rs).

## Steamroller Mode

`GameMode::Steamroller` (menu entry "Steamroller Mode") is a survival mode where a one-cell-thick wall line — the roller — sweeps back and forth across the arena:

- **Sweep:** `manage_steamroller` keeps the roller as a full line of cells (`press_cells`, mirrored into `obstacles`) at column `press_pos`. It starts at column 1 heading right (`press_dir = 1`) and advances one column per move, ping-ponging at the playable edges (`1..=width-2`; the border ring is already lethal). Moves are counted in `press_moves` and shown as `Roller: <pos><dir> | Moves: <count>` in the status bar, with a "The roller advances! (+N)" floating text. The line renders as a dark-red `=` (terminal) / dark-red cells (web).
- **Schedule:** within each 250-tick cycle the roller moves every 4th tick below tick 60, every 3rd tick below 120, every 2nd tick below 160, and every tick from 160 on — it visibly speeds up as the cycle wears on.
- **Reset:** at tick 200 of every 250-tick cycle the roller vacates all its cells and returns home (`press_pos = 1`, `press_dir = 1`, "The roller resets!").
- **Fairness:** a roller cell is never placed on a snake's body, on the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, and all hazard tile sets). The column the roller just left is vacated, so the sweeping wall can never crush or seal a snake in.
- **Autopilot:** roller cells are ordinary obstacles, so they are hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance — the bot rides out the roller natively.
- **Breaching:** eating food removes the roller cells adjacent to the eaten cell ("Breached N roller!").
- **Persistence:** `press_cells`, `press_pos`, `press_dir`, and `press_moves` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); stale roller records are pruned each tick if another effect cleared the obstacles.
- **Tests:** see [tests/test_steamroller_mode.rs](../../tests/test_steamroller_mode.rs) and the save/load test in `src/game/tests.rs`.

## Spike Pit Mode

`GameMode::SpikePit` (menu entry "Spike Pit Mode") is a survival mode with permanent spike vents and telegraphed, scheduled eruptions:

- **Vents:** `SPIKE_PIT_VENTS` (4) spike vents (`spike_vents`) are seeded on random fair cells at reset (`spawn_spike_vents`), at least 5 cells (Manhattan) from the spawn point, never on an obstacle, mine, food, or snake. `manage_spike_pit` re-seeds missing vents every tick so the pit stays armed, and the weather is pinned to `Weather::Clear` (ambient weather randomization is skipped in this mode).
- **Telegraph:** on every tick where `tick_counter % 6 == SPIKE_TELEGRAPH_TICKS (3)`, the cells that will erupt on the next multiple of 6 are recorded in `spike_warnings` (rendered as yellow `'` cracks terminal / yellow cells web) if they pass the fairness check.
- **Eruption:** on every 6th tick each vent pops spikes on itself and its four orthogonal neighbors (`spike_tiles`: cell → remaining up ticks, counted in `spike_eruptions`, "Spikes erupt! (+N)" floating text). Spikes are mirrored into `obstacles`, so they are lethal on entry and hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and trap avoidance — the bot rides out the pit natively. Re-erupting on an already-spiked cell just refreshes its timer.
- **Retraction:** each spike counts down one tick at a time and retracts after `SPIKE_UP_TICKS` (6) ticks, leaving the cell open (removed from both `spike_tiles` and `obstacles`); at tick 200 of every 250-tick cycle all spikes and warnings retract at once ("The spikes retract!").
- **Fairness:** an eruption never targets off-board cells, a snake's body, the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or any entity/obstacle (`is_spike_eruption_blocked`).
- **Eating food:** retracts the spikes adjacent to the eaten cell ("Retracted N spikes!").
- **Persistence:** `spike_tiles` (serialized through `map_as_pairs`), `spike_vents`, `spike_warnings`, and `spike_eruptions` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent); an end-of-tick self-heal drops stale spike records if something else cleared obstacles mid-tick. The status bar shows `Spikes: <count> | Eruptions: <count>`.
- **Tests:** see [tests/test_spike_pit_mode.rs](../../tests/test_spike_pit_mode.rs) and the save/load test in `src/game/tests.rs`.

## Landslide Mode

`GameMode::Landslide` (menu entry "Landslide Mode") is a survival mode wrapped in a permanent snowstorm (`Weather::Snow` forced every tick; ambient weather randomization is skipped for this mode):

- **Scheduled rolls:** on every 6th tick `manage_landslide` spawns a `LandslideRock` at a random board edge (`x = 0` rolling `Right`, or `x = width - 1` rolling `Left`) on a fairness-checked row, counted in `landslide_rolls` with an "Landslide!" floating text. On top of the schedule, a 5% ambient rumble per tick may roll one in, capped at `MAX_ROCKS = 3` concurrent rocks (the cap never blocks the schedule).
- **Rolling:** each tick every rock advances one cell in its direction; it smashes through obstacles and mines (`remove_obstacle_and_hazards`) and is removed when it rolls off the board.
- **Fairness:** `rock_safe_rows` never returns a row any snake currently occupies (body cells), a row containing the cell any snake is about to enter this tick (the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or a row that already has a rock.
- **Lethality:** a rock landing on a snake's head or any body cell kills that snake — the player/player 2 via the `hit_rock` checks alongside the meteor checks in the post-move collision pass, bots via `bots_to_remove`.
- **Autopilot:** `rock_position_after` predicts the exact cell a rock will occupy after `steps` ticks; `rock_would_hit` is a hard constraint in `is_safe_final_p`, the predicted rock lane is swept against the moving body in `path_is_safe`, rock cells join every hazard avoid-list next to the meteors, and the A* heuristic (`rock_lane_penalty` plus the row-ahead penalty in both heuristic sites) and edge costs (`rock_edge_cost`) keep the bot out of the rock's row — so the bot dodges landslides natively.
- **Bracing:** eating food removes the rock nearest to the eaten cell ("Braced!").
- **Rendering:** rocks draw as a white `>` / `<` arrow telegraphing the roll direction (terminal) and as white cells (web); the status bar shows `Rocks: <count> | Slides: <count>`.
- **Persistence:** `rocks` and `landslide_rolls` are stored in `SaveState` and `HistoryState` (save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_landslide_mode.rs](../../tests/test_landslide_mode.rs).
## Phase Shift Mode

`GameMode::PhaseShift` (menu entry "Phase Shift Mode") is a survival mode where two vertical gate walls phase between solid and ghost:

- **Gates:** `spawn_phase_gates` seeds two permanent vertical gate walls (`phase_gates`) at columns `width/3` and `2*width/3`, each spanning the playable rows `1..=height-2` except a 2-row gap, with the weather pinned to `Weather::Clear` (ambient weather randomization is skipped in this mode).
- **Phasing:** every `PHASE_PERIOD` (12) ticks `manage_phase_shift` toggles `phase_solid` and counts the flip in `phase_shifts` ("The gates phase!"). While solid, every gate cell is mirrored into `obstacles` (lethal on entry); while ghost, the cells are passable. The gates render as a magenta `#` (solid) / dark-magenta `.` (ghost) on the terminal and magenta / dark-magenta cells on the web.
- **Fairness:** a gate never solidifies on a snake's body, on the cell any snake is about to enter this tick (`phase_next_cells`: the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or on any entity (`phase_gate_clear`), so the phase flip can never kill anyone cheaply.
- **Autopilot:** the gates are treated as permanent walls in `is_safe_final_p` and in the `trap_blocked_cells` flood fill, so the bot never routes through a gate regardless of the current phase.
- **Disruption:** eating food removes the gate cells adjacent to the eaten cell from the solid set ("Disrupted N gate!").
- **Persistence:** `phase_gates`, `phase_solid`, and `phase_shifts` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent). The status bar shows `Gates: <count> | Phase: <phase> | Shifts: <count>` (`phase_str`).
- **Tests:** see [tests/test_phase_shift_mode.rs](../../tests/test_phase_shift_mode.rs).

## Trampoline Mode

`GameMode::Trampoline` (menu entry "Trampoline Mode", entry 85 in the main menu) is a survival mode played among a permanent set of launch pads:

- **Launch pads:** on reset `tramp_pad_count()` = `clamp(width*height/60, 3, 8)` pads (`tramp_pads`) are seeded on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another bumper, or the food. Pads are walkable (they are *not* mirrored into `obstacles`) and render as a yellow `+` (terminal) / yellow cell (web), white while overcharged.
- **Fling:** when a snake's next head cell is a pad, `tramp_launch` flings the head up to `TRAMP_LAUNCH` (3) cells — or `TRAMP_LAUNCH_CHARGED` (5) while overcharged — along its current direction, and counts it in `tramp_flings` ("Fling!"). The launch is applied to the player, player 2, and every bot at the same points where `belt_shove` is applied in `update`. The snake's direction is left unchanged, exactly like a belt shove.
- **Fairness:** only whole paths whose every cell passes `tramp_cell_free` are accepted, longest first — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column and every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble) and the Battle Royale safe zone. If no path is fair the snake simply rests on the walkable pad, so **a pad can never directly kill**.
- **Scheduled shuffle:** every 24th tick `manage_trampoline` relocates every pad to a fresh fair cell (`tramp_shuffles += 1`, "The pads rearrange! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Pads that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Overcharge:** eating food sets `tramp_overcharge = TRAMP_OVERCHARGE_TICKS` (12) ("Overcharged!"); while the timer is positive it counts down once per tick and flings go up to `TRAMP_LAUNCH_CHARGED` (5) cells.
- **Autopilot:** fling landings are always fair, so a launched bot is never killed by the launch itself; the bot replans from its actual head every tick and rides the pads natively.
- **Status bar:** `Pads: <count> | Flings: <count> | Shuffles: <count>` plus `Overcharged: <ticks>` while overcharged (`tramp_str`).
- **Persistence:** `tramp_pads`, `tramp_flings`, `tramp_shuffles`, and `tramp_overcharge` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_trampoline_mode.rs](../../tests/test_trampoline_mode.rs).

## Polarity Mode

`GameMode::Polarity` (menu entry "Polarity Mode", entry 86 in the main menu) is a survival mode played among a permanent set of magnetic poles whose polarity flips on a schedule:

- **Magnetic poles:** on reset `polarity_pole_count()` = `clamp(width*height/80, 2, 6)` poles (`polarity_poles`) are seeded on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another mode's fixture, or the food. Poles are walkable (they are *not* mirrored into `obstacles`) and render as a red `M` while attracting / blue `M` while repelling (terminal) and red / blue cells (web).
- **Polarity flip:** every `POLARITY_FLIP_PERIOD` (16) ticks `manage_polarity` toggles `polarity_attract` and counts it in `polarity_flips` ("Polarity flipped: Attract!" / "Repel!"). The field starts attracting on reset.
- **Shove:** on a flip tick, `polarity_shove` shoves the head one cell toward the nearest pole (Manhattan distance) while attracting, or away from it while repelling — dominant axis first, then the other axis — and counts it in `polarity_shoves` ("Pulled!" / "Pushed!"). The shove is applied to the player, player 2, and every bot at the same points where `belt_shove`/`tramp_launch` are applied in `update`, and only on flip ticks (never when the snake hit a wall).
- **Fairness:** only destination cells passing `polarity_cell_free` are accepted — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column, every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble) and the Battle Royale safe zone. If no candidate cell is fair the snake is not shoved at all, so **a pole can never directly kill**.
- **Shield:** eating food sets `polarity_shield = POLARITY_SHIELD_TICKS` (10) ("Shielded!"); while the timer is positive it counts down once per tick and the player is not shoved.
- **Scheduled shuffle:** every `POLARITY_SHUFFLE_PERIOD` (24) ticks `manage_polarity` relocates every pole to a fresh fair cell (`polarity_shuffles += 1`, "The poles rearrange! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Poles that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Autopilot:** shove landings are always fair, so a shoved bot is never killed by the shove itself; the bot replans from its actual head every tick and rides the poles natively.
- **Status bar:** `Poles: <count> | Polarity: <Attract/Repel> | Flips: <count> | Shoves: <count> | Shuffles: <count>` plus `Shielded: <ticks>` while shielded (`polarity_str`).
- **Persistence:** `polarity_poles`, `polarity_attract`, `polarity_flips`, `polarity_shuffles`, `polarity_shoves`, and `polarity_shield` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_polarity_mode.rs](../../tests/test_polarity_mode.rs).

## Blink Mode

`GameMode::Blink` (menu entry "Blink Mode", entry 87 in the main menu) is a survival mode played among a permanent set of teleport pads:

- **Blink pads:** on reset `blink_pad_count()` = `clamp(width*height/60, 3, 8)` pads (`blink_pads`) are seeded on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another pad, or the food. Pads are walkable (they are *not* mirrored into `obstacles`) and render as a cyan `*` (terminal) / cyan cell (web), white while overcharged.
- **Blink:** when a snake's next head cell is a pad, `blink_teleport` moves the head to a random fair cell and counts it in `blink_teleports` ("Blink!"). The blink is applied to the player, player 2, and every bot right after the `belt_shove`/`tramp_launch`/`polarity_shove` step in `update`. The snake's direction is left unchanged.
- **Fairness:** the destination must pass `blink_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`, via `tramp_cell_free`), plus the lightning column, every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble), the Battle Royale safe zone, the food, and other blink pads. If no fair destination exists the snake simply rests on the walkable pad, so **a pad can never directly kill**.
- **Scheduled scatter:** every `BLINK_PERIOD` (24) ticks `manage_blink` relocates every pad to a fresh fair cell (`blink_shuffles += 1`, "The pads scatter! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Pads that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Overcharge:** eating food sets `blink_overcharge = BLINK_OVERCHARGE_TICKS` (12) ("Overcharged!"); while the timer is positive it counts down once per tick and each blink is biased to a fair cell adjacent to the food (falling back to any fair cell when none is adjacent).
- **Autopilot:** blink landings are always fair, so a blinked bot is never killed by the blink itself; the bot replans from its actual head every tick and rides the pads natively.
- **Status bar:** `Pads: <count> | Blinks: <count> | Shuffles: <count>` plus `Overcharged: <ticks>` while overcharged (`blink_str`).
- **Persistence:** `blink_pads`, `blink_teleports`, `blink_shuffles`, and `blink_overcharge` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_blink_mode.rs](../../tests/test_blink_mode.rs).
## Crossfire Mode

`GameMode::Crossfire` (menu entry "Crossfire Mode", entry 123 in the main menu) is a survival mode built around scheduled **cross beams**:

- **Scheduling:** every `CROSSFIRE_PERIOD` (6) ticks `manage_crossfire` picks a cross — one column and one row (`crossfire_target: Option<(u16, u16)>`, column first) — via `pick_safe_cross`.
- **Fairness:** `pick_safe_cross` never picks a column or row any snake currently occupies (head *or* body, since a beam sweeps the whole line) or is about to enter this tick (the player's and player 2's current direction is the one they will move in this tick; bots use their queued direction), so a cross can never be scheduled on top of a snake. Lines are drawn from the playable area `1..width-1` / `1..height-1` (honoring the Battle Royale safe-zone margin).
- **Telegraph:** the cross is telegraphed for `CROSSFIRE_TELEGRAPH_TICKS` (3) ticks (`crossfire_warn_ticks`), rendered as a yellow `-` (row) / `|` (column) / `+` (intersection) on the terminal and yellow lines on the web UI.
- **Beam:** when the telegraph hits zero the beam fires for `CROSSFIRE_BEAM_TICKS` (2) ticks (`crossfire_beam_ticks`, counted in `crossfire_strikes`, "Crossfire!"), rendered red (`=` / `|` / `X`). `crossfire_beam_cell` marks beam cells lethal: the player, player 2, and every bot whose final head lands on the beam column or row dies.
- **Reset:** at tick 200 of every 250-tick cycle the crossfire dies down ("The crossfire dies down!"). Eating food **dampens** it ("Dampened!"), instantly clearing the telegraph and any active beam. The weather is pinned to `Weather::Clear` so the cross is always visible.
- **Autopilot:** `crossfire_danger_cell` (telegraphed or active) is a hard hazard in `is_safe_final_p` and `trap_blocked_cells`, a heavy penalty in the `astar_search` and `get_boss_path` heuristics/edge costs, and a hard reject in the gust/belt/singularity/minefield destination checks and `tramp_cell_free`, so the bot never routes onto a cross line.
- **Status bar:** `Cross: C<col>/R<row> (Warn <ticks>/Beam <ticks>) | Strikes: <count>` (`crossfire_str`).
- **Persistence:** `crossfire_target`, `crossfire_warn_ticks`, `crossfire_beam_ticks`, and `crossfire_strikes` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_crossfire_mode.rs](../../tests/test_crossfire_mode.rs).

## Quicksand Mode

`GameMode::Quicksand` (menu entry "Quicksand Mode") is a survival mode played on a board that is slowly being swallowed by a spreading bog. The weather is pinned to `Weather::Rain` every tick (ambient weather randomization is skipped in this mode).

- **Patches:** at reset `spawn_initial_quicksand` seeds `QUICKSAND_INITIAL_PATCHES` (3) patches (`quicksand`): each is a fair center cell at least `QUICKSAND_MIN_SPAWN_DIST` (5) cells (Manhattan) from the spawn plus up to `QUICKSAND_PATCH_CELLS` (4) fair orthogonal arms. Quicksand is walkable (it is *not* mirrored into `obstacles`) and renders as a dark-yellow `~` (terminal) / brown cells (web). The status bar shows `Bog: <cells> | Spreads: <n> | Sink: <t>/<QUICKSAND_SINK_TICKS>` (`quicksand_str`).
- **Spreading:** on every `QUICKSAND_PERIOD` (8th) tick `manage_quicksand` claims new cells (counted in `quicksand_spreads`, "The bog spreads! (+N)"): with 50% chance (always while the bog is empty) it seeds a fresh patch whose center is at least 3 cells (Manhattan) from every snake head, otherwise it grows a random existing patch by up to 2 cells. Total bog is capped at `QUICKSAND_MAX_CELLS` (36) cells.
- **Fairness:** a bog cell is never claimed off-board, on an obstacle, mine, or any entity or hazard tile, on a snake's body, on the cell any snake is about to enter this tick (`is_quicksand_cell_blocked` with the next-head cells: the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), or on an already-boggy cell — so the bog can never appear under anyone.
- **Sinking:** quicksand is walkable but sticky: each tick a snake whose head ends the tick on a bog cell sinks one step (`quicksand_sink` / `quicksand_sink2` for the players, `quicksand_bot_sinks` per bot); leaving the bog resets the counter to 0. After `QUICKSAND_SINK_TICKS` (12) consecutive ticks in the bog the snake is swallowed and dies (players via the `swallowed` checks next to the thorn ones, bots via `bots_to_remove`, with dark-yellow `~` particles).
- **Drain:** at tick 200 of every 250-tick cycle the whole bog drains at once ("The bog drains!"): all cells are removed and every sink counter resets.
- **Stomping:** eating food removes the bog cells adjacent to the eaten cell and resets the eater's sink counter ("Stomped N bog!").
- **Autopilot:** bog cells are hard constraints in the A* search, the `is_safe_final_p` verified-safe path checks, and the `trap_blocked_cells` flood fill (the bot never plans a route through quicksand), so it rides out the bog natively.
- **Persistence:** `quicksand`, `quicksand_spreads`, `quicksand_sink`, `quicksand_sink2`, and `quicksand_bot_sinks` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_quicksand_mode.rs](../../tests/test_quicksand_mode.rs) and the save/load test in `src/game/tests.rs`.

## Domino Mode

`GameMode::Domino` (menu entry "Domino Mode", entry 126 in the main menu) is a survival mode played among standing domino obstacles that periodically topple in chains:

- **Dominoes:** on reset `domino_count()` = `clamp(width*height/80, 4, 10)` dominoes (`domino_tiles`) are seeded on fair cells at least 4 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another domino, or the food. Dominoes are mirrored into `obstacles`, so entering one kills the snake, and they render as a dark-yellow `D` (terminal) / goldenrod cell (web).
- **Topple wave:** every `DOMINO_PERIOD` (8) ticks `manage_domino` fires a wave (`domino_waves += 1`): a random standing domino is picked and knocked in a random direction. The whole chain of dominoes along that line (up to `DOMINO_CHAIN_MAX` = 6) is shoved one cell forward — the first cell is vacated and the last domino lands on the first non-domino cell ("The dominoes topple! (N)", counted in `domino_falls`). A wave shoves dominoes; it never adds new ones.
- **Fairness:** the landing cell must pass `is_domino_landing_blocked` — in bounds within the safe-zone margin and free of snake bodies, the cells snakes are about to enter this tick (player/player 2 use their current direction, bots their queued direction), every entity, and every hazard tile set. Intermediate chain cells were already obstacles and remain obstacles, so only the landing cell can create new danger. If no fair landing exists the wave fizzles and nothing moves, so **a wave can never crush a snake**.
- **Scheduled drops:** every `DOMINO_DROP_PERIOD` (24) ticks a fresh domino drops on a fair cell while the field is under the cap of 15% of the board (`domino_drops += 1`, "A domino drops!"); if the field is ever empty, the next wave stands one fresh domino up instead of toppling.
- **Stand-again:** at tick 200 of every 250-tick cycle all dominoes are gathered up (removed from `domino_tiles` and `obstacles`) and re-seeded on fresh fair cells at least 4 cells (Manhattan) from any snake head (`domino_reshuffles += 1`, "The dominoes stand again! (N)").
- **Bracing:** eating food removes the dominoes orthogonally adjacent to the eaten cell from `domino_tiles` and `obstacles` ("Braced N domino(es)!").
- **Autopilot:** because dominoes live in `obstacles`, the bot treats them as hard walls in the A* search, the verified-safe `is_safe_final_p` checks, and the trap-avoidance flood fill — it routes around them natively, and wave landings are always fair, so a toppled field can never kill the bot cheaply.
- **Weather:** pinned to `Weather::Clear` (and excluded from the random-weather roll) so the dominoes always stay visible.
- **Status bar:** `Dominoes: <count> | Waves: <count> | Falls: <count> | Reshuffles: <count>` (`domino_str`).
- **Persistence:** `domino_tiles`, `domino_waves`, `domino_falls`, `domino_drops`, and `domino_reshuffles` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_domino_mode.rs](../../tests/test_domino_mode.rs).
## Magnet Mode

`GameMode::Magnet` (menu entry "Magnet Mode", entry 126 in the main menu) is a survival mode played among a permanent set of magnetic poles:

- **Magnetic poles:** on reset `magnet_pole_count()` = `clamp(width*height/150, 2, 6)` poles (`magnet_poles: HashMap<Point, bool>`, `true` = attract, `false` = repel) are seeded by `spawn_magnet_poles` on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, on another pole, or on an obstacle. Poles are walkable markers (they are *not* mirrored into `obstacles`) and render as a cyan `+` (attract) / red `-` (repel) on the terminal and cyan / red cells on the web.
- **Pulse:** every `MAGNET_PULSE_PERIOD` (6) ticks `manage_magnet_pulse` (called at the end of `update`, right after `manage_tornado_gust`) fires a board-wide pulse counted in `magnet_pulses`. Each snake — the player, player 2, and every bot, sequentially so earlier pulses are respected by later checks — is shoved one cell toward (attract) or away from (repel) its nearest pole along the dominant axis (`magnet_pulse_step`) with `move_to(dest, false)`, so a pulse never grows the snake ("Pulled!"/"Pushed!").
- **Fairness:** `magnet_destination` wraps `gust_destination` (in bounds within the safe-zone margin unless the board wraps; no obstacles, lava, mines, black hole, lightning column, bosses, lasers, meteors, rocks, turrets, tornado, or any snake body) and additionally rejects the remaining lethal tile sets (fire, acid, puddles, inferno, blaze, seismic rubble), portals, the cells the *other* snakes are about to enter, and any cell a falling meteor will hit on the next tick (`meteor_would_hit(m, dest, 1)`). If no fair destination exists the pulse fizzles for that snake, so **a pulse can never directly kill**.
- **Scheduled relocation:** every `MAGNET_RELOCATE_PERIOD` (24) ticks `manage_magnet` relocates every pole to a fresh fair cell with a re-rolled polarity (`magnet_relocations += 1`, "The poles shift! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Poles that end up under a fresh obstacle (e.g. a meteor crash) are demagnetized at the start of each tick.
- **Polarity reversal:** eating food flips the polarity of the pole nearest the eaten cell (`magnet_flips += 1`, "Reversed!").
- **Autopilot:** pulse destinations are always fair, so a shoved bot is never killed by the pulse itself; the bot replans from its actual head every tick and rides the pulses natively.
- **Status bar:** `Poles: <count> | Pulses: <count> | Relocations: <count> | Flips: <count>` (`magnet_str`).
- **Persistence:** `magnet_poles`, `magnet_pulses`, `magnet_relocations`, and `magnet_flips` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_magnet_mode.rs](../../tests/test_magnet_mode.rs).
## Gale Mode

`GameMode::Gale` (menu entry "Gale Mode", entry 127 in the main menu) is a survival mode played under a board-wide wind:

- **Wind:** `gale_dir` is randomized at reset. After every snake's regular move (player, player 2, and every bot — applied right where `belt_shove`/`tramp_launch` are applied in `update`), `gale_shove` pushes the head one extra cell along `gale_dir` and counts it in `gale_pushes` ("Gust!").
- **Fairness:** a gust only fires when the sinkholeget cell is in bounds, passes `tramp_cell_free` (in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter via `singularity_cell_free`, plus obstacles, food, mines, the black hole, portals, entities, the lightning column, and every lethal tile set), lies outside every falling meteor's lane (`gale_meteor_lane`), and is not claimed by another snake's head this tick (`gale_claimed`, seeded from `singularity_next_heads` and extended with every final head). If the sinkholeget is not fair the snake simply braces in place, so **a gust can never push anyone into a kill or into another head**.
- **Scheduled shifts:** every `GALE_PERIOD` (18) ticks `manage_gale` turns the wind to a new random direction different from the current one (`gale_shifts += 1`, "The wind shifts <dir>!").
- **Bracing:** eating food sets `gale_braces = GALE_BRACE_TICKS` (10) ("Braced!"); while the timer is positive it counts down once per tick in `manage_gale` and the player is never pushed (`gale_shove` returns `None` for a braced snake).
- **Autopilot:** gust landings are always fair, so a pushed bot is never killed by the gust itself; the bot replans from its actual head every tick and rides the wind natively.
- **Status bar:** `Wind: <dir> | Pushes: <count> | Shifts: <count>` plus `Braced: <ticks>` while braced (`gale_str`); a compass arrow renders in the top-right playable cell (dark-cyan `^`/`v`/`<`/`>` on the terminal, white while braced; dark-cyan arrow text on the web UI).
- **Persistence:** `gale_dir`, `gale_pushes`, `gale_shifts`, and `gale_braces` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_gale_mode.rs](../../tests/test_gale_mode.rs).

## Crumble Mode

`GameMode::Crumble` (menu entry "Crumble Mode", entry 88 in the main menu) is a survival mode played over a floor of cracked tiles — each crack is a one-time bridge:

- **Cracked floor:** on reset `spawn_crumble_cracks` seeds `clamp(width*height/50, 4, 10)` cracks (`crumble_tiles`) on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a hazard tile, or the food. Cracks render as a dark-yellow `:` (terminal) / brown cell (web). The weather is pinned to `Weather::Clear` so the cracks are always visible.
- **Collapse:** a crack is walkable, but when a snake head leaves it, `manage_crumble` (at the start of the tick, before movement) turns the tile into a permanent `obstacles` pit and counts it in `crumble_collapses` ("Crumble!"). Obstacle death is only ever checked on the *new* head, so the mover can never be crushed by its own trail.
- **Fairness:** a crack never collapses while any snake is about to enter its cell (`singularity_next_heads`) or while an entity (food, bonus/poison food, power-up, mine, portal, black hole, tornado) sits on it (`crumble_cell_claimed`), so a collapse can never kill anyone cheaply.
- **Scheduled cracking:** every `CRUMBLE_PERIOD` (8) ticks up to `CRUMBLE_NEW_PER_PERIOD` (2) fresh cracks appear on fair cells (`crumble_candidates` / `crumble_cell_free`), counted in `crumble_cracks` ("The floor cracks! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, within 3 cells of any snake head, and capped at `clamp(width*height/40, 6, 16)` live cracks.
- **Periodic fill:** at tick 200 of every 250-tick cycle all remaining cracks fill back in without ever becoming pits ("The cracks fill in!").
- **Reinforcement:** eating food removes the cracks adjacent (4-neighborhood) to the eaten cell ("Reinforced N crack(s)!").
- **Autopilot:** cracks are treated as permanent walls in `is_safe_final_p` and `trap_blocked_cells`, so the bot never routes over a crack in the A* search, the verified-safe path checks, or the trap-avoidance flood fill.
- **Status bar:** `Cracks: <count> | Collapses: <count> | New: <count>` (`crumble_str`).
- **Persistence:** `crumble_tiles`, `crumble_collapses`, and `crumble_cracks` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_crumble_mode.rs](../../tests/test_crumble_mode.rs).

## Comet Mode

`GameMode::Comet` (menu entry "Comet Mode", entry 132 in the main menu) is a survival mode where comets streak diagonally across a permanently clear sky:

- **Launch:** every `COMET_PERIOD` (18) ticks `manage_comet` guarantees a scheduled launch (`comet_launches`, "Comet!"), plus a rare ambient streak (2% per tick), capped at `MAX_COMETS` (2) concurrent comets. `launch_comet` picks a random edge cell and an inward diagonal `(dx, dy)` (±1, ±1) and only accepts it when `comet_path_fair` confirms the whole diagonal avoids every snake body segment and the cell every snake is about to enter — so a launch is dangerous but never cheap.
- **Flight and trail:** each comet moves one cell per tick along its diagonal until it leaves the board. Every cell it touches is ignited via `comet_ignite` into `fire_tiles` + `fire_ttl` (`COMET_TAIL_TICKS` = 12) and mirrored into `obstacles`, so the burning trail is lethal on entry and a hard obstacle in the A* search and the verified-safe path checks; the head cell's timer is refreshed every tick while the comet occupies it. Burned-out cells cool off (removed from `fire_tiles`/`fire_ttl`/`obstacles`) each tick. A comet smashes through obstacles and mines on its path and kills any snake it touches (`hit_comet1`/`hit_comet2` and the bot kill checks in `update_tick`).
- **Sky clear:** at tick 200 of every 250-tick cycle all comets burn out ("The sky clears!").
- **Deflect:** eating food removes the comet nearest to the eaten cell (`comet_deflections`, "Deflected!").
- **Autopilot:** `comet_position_after` / `comet_would_hit` predict the exact diagonal, so predicted comet cells are hard hazards in `is_safe_final_p`, in the body-sweep check of `path_is_safe`, in the A* lane penalties (`comet_lane_penalty`) and edge costs (`comet_edge_cost`, in both `astar_search` and `get_boss_path`), in the boss `can_move` checks, and in the `trap_blocked_cells` flood fill — the bot dodges comets and their trails natively.
- **Status bar:** `Comets: <count> | Launches: <count> | Deflects: <count>` (`comet_str`). Comet heads render as a yellow `*` (terminal) / gold cell (web) over the red burning trail.
- **Persistence:** `comets`, `comet_launches`, and `comet_deflections` are stored in `SaveState` and `HistoryState` (the trail rides along in the already-persisted `fire_tiles`/`fire_ttl`/`obstacles`), so save/load and time-rewind stay consistent.
- **Tests:** see [tests/test_comet_mode.rs](../../tests/test_comet_mode.rs).

## Elevator Mode

`GameMode::Elevator` (menu entry "Elevator Mode", entry 130 in the main menu) is a movement mode played among a permanent set of elevator shafts:

- **Shafts:** on reset `elev_shaft_count()` = `clamp(width*height/160, 2, 5)` shafts (`elev_shafts`, each an ordered `(a, b)` pair of cells sharing one column, `ELEV_MIN_GAP`..=`ELEV_MAX_GAP` = 4..=10 rows apart) are seeded by `place_elev_shafts` on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, on the cell any snake is about to enter, on any entity, an obstacle, a mine, a portal, the black hole, the tornado, a hazard tile, or the food. Shaft ends are walkable (they are *not* mirrored into `obstacles`) and render as a cyan `|` (terminal) / cyan cell (web), dark cyan while the shaft is recharging, white while overcharged. The weather is pinned to `Weather::Clear` so the lifts stay visible.
- **Ride:** when a snake's next head cell is one end of a shaft, `elev_ride` carries the head to the other end and counts it in `elev_rides` ("Ride!"). The ride is applied to the player, player 2, and every bot in `update` right after the belt/tramp/polarity/blink steps. The snake's direction is left unchanged.
- **Fairness:** the destination must pass `elev_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column and every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble) and the Battle Royale safe zone — and the shaft must not be recharging (`elev_cooldowns[idx] == 0`). If no ride is fair the snake simply rests on the walkable shaft end, so **a lift can never directly kill**.
- **Cooldown:** after a ride the shaft recharges for `ELEV_COOLDOWN` (6) ticks (`elev_cooldowns[idx]`), counted down once per tick in `manage_elevator`; a recharging shaft never fires.
- **Scheduled relocation:** every `ELEV_SHUFFLE_PERIOD` (20) ticks `manage_elevator` relocates every shaft to fresh fair cells (`elev_shuffles += 1`, "The elevators shift! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, on another shaft, or within 3 cells (Manhattan) of any snake head. Shafts with an end under a fresh obstacle or lethal tile are scrapped at the start of each tick.
- **Overcharge:** eating food sets `elev_overcharge = ELEV_OVERCHARGE_TICKS` (12) ("Overcharged!"); while the timer is positive it counts down once per tick and rides ignore cooldowns (and do not refresh them).
- **Autopilot:** ride destinations are always fair, so a lifted bot is never killed by the ride itself; the bot replans from its actual head every tick and rides the lifts natively.
- **Status bar:** `Lifts: <count> | Rides: <count> | Shifts: <count>` plus `Overcharged: <ticks>` while overcharged (`elev_str`).
- **Persistence:** `elev_shafts`, `elev_cooldowns`, `elev_rides`, `elev_shuffles`, and `elev_overcharge` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_elevator_mode.rs](../../tests/test_elevator_mode.rs).

## Mire Mode

`GameMode::Mire` (menu entry "Mire Mode", entry 131 in the main menu) is a survival mode played on treacherous ground. The weather is pinned to `Weather::Clear` (the ambient weather re-roll skips this mode) so the cracks and pools are always visible:

- **Telegraph:** every `MIRE_PERIOD` (6) ticks, `MIRE_TELEGRAPH_TICKS` (3) ticks before the wave (tick % 6 == 3), `manage_mire` clears `mire_warnings` and cracks up to `MIRE_PER_WAVE` (3) fresh cells chosen at random from the fair candidate cells, at least 5 cells (Manhattan) from the spawn. On reset, `spawn_mire_warnings` seeds `MIRE_INITIAL_WARNINGS` (4) such cracks.
- **Scheduled wave:** every 6th tick the warned cells sink: each becomes a mire pool (`mire_cells`, cell -> remaining ticks) and is mirrored into `obstacles`, so entering one kills through the standard obstacle path. Waves are counted in `mire_waves` ("The mire claims N cell(s)!").
- **Fairness:** a cell never sinks on a snake's body, on the cell any snake is about to enter this tick (player/player 2 use their current direction, bots their queued direction), or on any entity or hazard tile (`mire_cell_blocked`) — so a wave can never cheaply kill anyone. Cracks render as dark-yellow `.` (terminal) / dark-brown cells (web); pools as yellow `~` / brown cells.
- **Drying:** each pool counts down one tick per tick and dries up after `MIRE_POOL_TICKS` (24) ticks, leaving open ground (removed from both `mire_cells` and `obstacles`). At tick 200 of every 250-tick cycle all mire dries up and all cracks clear ("The mire settles!"). If something else clears obstacles mid-tick (e.g. a Bomb), stale mire records are pruned on the next tick so the mirror stays exact.
- **Eating food:** stirs the mire — every pool orthogonally adjacent to the eaten cell dries up ("Stirred N pool!").
- **Autopilot:** `is_safe_final_p` rejects both active pools and telegraphed cracks as hard constraints, so the bot never plans onto ground that is up or about to give way.
- **Status bar:** `Mire: <count> | Waves: <count>` (`mire_bar`).
- **Persistence:** `mire_cells`, `mire_warnings`, and `mire_waves` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_mire_mode.rs](../../tests/test_mire_mode.rs).

## Tar Pit Mode

`GameMode::TarPit` (menu entry "Tar Pit Mode", entry 132 in the main menu) is a survival mode played on a board with spreading lethal tar pits:

- **Seeding:** on reset `spawn_tar_patches` seeds `TAR_PATCHES` (3) plus-shaped pits: each pit center goes into `tar_patches`, and the center plus its four in-bounds neighbors become active tar (`tar_pits`, mirrored into `obstacles`) on fair interior cells at least 5 cells (Manhattan) from the spawn. The weather is pinned to `Weather::Clear` (ambient weather randomization is skipped in this mode).
- **Schedule:** every `TAR_PERIOD` (6) ticks `manage_tar_pit` runs a cycle. At tick `TAR_TELEGRAPH_TICKS` (3) mod 6 the threatened cells are telegraphed into `tar_warnings` (loose tar, walkable, rendered as yellow `.` / yellow cells): a fresh plus-shaped pit is seeded every 12th tick while under `TAR_MAX_PATCHES` (6) centers and `TAR_MAX_CELLS` (80) active cells, and the existing pits creep outward by up to `TAR_SPREAD_CELLS` (2) cells per pit per cycle from the sorted tar frontier.
- **Activation:** at every 6th tick the telegraphed cells activate into `tar_pits` + `obstacles` (counted in `tar_spreads`, "Tar spreads! (+N)"), capped at `TAR_MAX_CELLS`. Tar is lethal on entry and a hard hazard in the A* search and the verified-safe `is_safe_final_p` checks (via the `obstacles` mirror), so the bot routes around it natively.
- **Fairness:** `is_tar_cell_blocked` never claims border-ring or off-board cells, a snake's body, the cell any snake is about to enter this tick (`phase_next_cells`: the player's and player 2's direction queues are popped before the manage calls, so their current direction is authoritative; the bots' queues are popped after, so their queued direction is used), the food, or any entity or hazard tile — and it is re-checked at activation time too, so spreading tar can never claim anyone cheaply.
- **Setting cycle:** at tick `TAR_DRY_TICK` (200) of every 250-tick cycle all tar, warnings, and pit centers are cleared at once ("The tar sets hard!").
- **Stomping:** eating food removes the tar cells adjacent to the eaten cell from `tar_pits` and `obstacles` ("Stomped N tar!") and drops pit centers that no longer have any tar.
- **Self-heal:** each tick `manage_tar_pit` (and an end-of-tick pass) drops stale tar records if something else cleared obstacles mid-tick, keeping the `tar_pits`/`obstacles` mirror exact.
- **Status bar:** `Tar: <count> | Pits: <count> | Spreads: <count>` (`tar_str`).
- **Persistence:** `tar_pits`, `tar_patches`, `tar_warnings`, and `tar_spreads` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_tar_pit_mode.rs](../../tests/test_tar_pit_mode.rs) and the save/load test in `src/game/tests.rs`.
## Faultline Mode

`GameMode::Faultline` (menu entry "Faultline Mode", entry 133 in the main menu) is a survival mode where the floor itself crumbles away under the snakes:

- **Crack waves:** on reset one wave of cracks (`fault_tiles: HashMap<Point, u32>`, each mapped to its remaining ticks) is seeded by `spawn_fault_wave(5)` on `clamp(width*height/60, 4, 10)` fair cells at least 5 cells (Manhattan) from the spawn; every `FAULT_PERIOD` (20) ticks `manage_crumble` seeds another wave (`fault_waves += 1`, "The floor cracks! (N)") at least 4 cells from every snake head. A crack never lands on a snake, on the cell any snake is about to enter (`fault_next_cells`: player/p2 use their current direction, bots their queued direction), on any entity, obstacle, mine, portal, pit, existing crack, food, power-up, hazard tile (fire, acid, puddles, lava, inferno, blaze, seismic rubble, ice, belts), or another mode's tile set.
- **Telegraph and collapse:** each crack ticks down once per tick; at zero it collapses into `fault_pits` (`fault_collapses += 1`), which is mirrored into `obstacles` every tick — so a pit kills on entry and is a hard wall for the A* search, `is_safe_final_p`, and the `trap_blocked_cells` flood fill. Cracks themselves are also hard walls for the autopilot in `is_safe_final_p` and `trap_blocked_cells`, so the bot never routes onto cracking floor.
- **Fairness:** `fault_collapse_blocked` never lets a crack collapse on a snake's body or on the cell any snake is about to enter this tick — the collapse is deferred (the crack stays armed at 1 tick and retries next tick), so **the floor can never collapse under anyone**. Cracks that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Heal:** every `FAULT_HEAL_PERIOD` (200) ticks the whole floor stabilizes (`fault_heals += 1`, "The floor heals!"): all pits leave the obstacle mirror and all cracks clear, after which the tick-200 crack wave (200 is a multiple of 20) re-cracks the floor.
- **Reinforcement:** eating food removes the cracked cells adjacent to the eaten cell (`fault_repairs += N`, "Repaired N tile(s)!").
- **Visibility:** the weather is pinned to `Weather::Clear` so cracks and pits are always visible; cracks render as a dark-yellow `.` and pits as a dark-blue `~` on the terminal, saddle-brown and royal-blue cells on the web.
- **Status bar:** `Cracks: <count> | Pits: <count> | Waves: <count> | Collapses: <count>` (`fault_str`).
- **Persistence:** `fault_tiles`, `fault_pits`, `fault_waves`, `fault_collapses`, `fault_heals`, and `fault_repairs` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_fault_mode.rs](../../tests/test_fault_mode.rs).
## Electro Mode

`GameMode::Electro` (menu entry "Electro Mode", entry 134 in the main menu) is a survival mode played among a permanent set of electromagnets. The weather is pinned to `Weather::Clear` so the magnets stay visible.

- **Electromagnets:** on reset `electro_magnet_count()` = `clamp(width*height/60, 3, 8)` magnets (`electro_magnets`) are seeded on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another pad/bumper, or the food. Magnets are walkable (they are *not* mirrored into `obstacles`) and render as a cyan `M` while attracting, a red `M` while repelling, and a dark-grey `M` while grounded (terminal) / cyan, red, or grey cells (web).
- **Pulse:** every `ELECTRO_PULSE_PERIOD` (6) ticks, `electro_drag` takes the snake's final head cell and steps one cell along the dominant axis (x on ties) toward the nearest active magnet (`electro_nearest_magnet`, Manhattan distance, ties broken by (x, y)) while `electro_attract` is true, or away from it while repelling, and counts the drag in `electro_pulses` ("Zap! Pulled!" / "Zap! Pushed!"). The pulse is applied to the player, player 2, and every bot right after `tramp_launch` in `update`. The snake's direction is left unchanged, exactly like a belt shove.
- **Fairness:** the drag destination must pass `tramp_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column and every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble) and the Battle Royale safe zone. If no step is fair the pulse simply fizzles and the normal move stands, so **a pulse can never directly kill**.
- **Electro flips:** every `ELECTRO_FLIP_PERIOD` (12) ticks `manage_electro` toggles `electro_attract` and counts the flip in `electro_flips` ("Electro: attract!" / "Electro: repel!"). The flip happens in the manage phase, before the pulse of the same tick, so tick 12 already pulses with the new electro.
- **Scheduled shuffle:** every `ELECTRO_SHUFFLE_PERIOD` (24) ticks `manage_electro` relocates every magnet to a fresh fair cell (`electro_shuffles += 1`, "The magnets rearrange! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Magnets that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Grounding:** eating food grounds the nearest active magnet to the eaten cell for `ELECTRO_GROUND_TICKS` (30) ticks (`electro_grounded` countdown, "Grounded!"); a grounded magnet is inert (excluded from `electro_active_magnets`) and reactivates when its timer reaches zero.
- **Autopilot:** pulse landings are always fair, so a dragged bot is never killed by the pulse itself; the bot replans from its actual head every tick and rides the magnets natively.
- **Status bar:** `Magnets: <count> | <Attract/Repel> | Pulses: <count> | Flips: <count> | Shuffles: <count>` plus `Grounded: <count>` while any magnet is grounded (`electro_str`).
- **Persistence:** `electro_magnets`, `electro_attract`, `electro_pulses`, `electro_flips`, `electro_shuffles`, and `electro_grounded` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_electro_mode.rs](../../tests/test_electro_mode.rs).

## Sludge Mode

`GameMode::Sludge` (menu entry "Sludge Mode", entry 135 in the main menu) is a survival mode where pools of lethal sludge keep appearing and creeping across the board:

- **Seeds:** on reset `spawn_initial_sludge` places `max(2, width*height/100)` sludge pools (`sludge_tiles`) on fair cells at least 5 cells (Manhattan) from the spawn point — never on a snake, an entity, an obstacle, or the food. Sludge is mirrored into `obstacles`, so it is lethal on entry and renders as a dark-grey `;` (terminal) / dark-slate cell (web).
- **Scheduled spills:** every `SLUDGE_SPILL_PERIOD` (6) ticks `manage_sludge` fires a spill (counted in `sludge_spills`, "Sludge spills! (+N)"): `spill_sludge` opens `SLUDGE_BURST` (3) fresh pools on random fair cells via `get_random_empty_point` and creeps the field up to `SLUDGE_SPREAD` (2) cells outward from a random frontier cell, capped at `SLUDGE_CAP_PERCENT` (12)% of the board.
- **Fairness:** `is_sludge_spill_blocked` never spills onto a cell that any snake currently occupies (head or body) or is about to enter this tick (player/p2 use their current direction, bots their queued direction), nor any entity (food, bonus/poison food, power-ups, portals, mines, black hole, tornado, goblin, bosses, companion, turrets, XP gems, painted tiles, equipment boxes, crops, eggs, the KoTH zone, flags, resources, lasers, meteors, rocks, the lightning column) or any other hazard tile set, so a spill can never kill anyone cheaply.
- **Settling:** at tick 200 of every 250-tick cycle the whole field settles (`sludge_settles += 1`, "The sludge settles! (N)"): every cell is removed from both `sludge_tiles` and `obstacles`. An end-of-tick self-heal drops stale sludge records if something else cleared obstacles mid-tick (e.g. a Bomb).
- **Autopilot:** because sludge lives in `obstacles`, the bot treats it as a hard obstacle in the A* search, the verified-safe `is_safe_final_p` checks, and the trap-avoidance flood fill natively, so it routes around the pool.
- **Counterplay:** eating food stirs the sludge adjacent to the eaten cell loose ("Stirred N sludge!"), removing those cells from both `sludge_tiles` and `obstacles`.
- **Status bar:** `Sludge: <count> | Spills: <count> | Settles: <count>` (`sludge_str`).
- **Persistence:** `sludge_tiles`, `sludge_spills`, and `sludge_settles` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_sludge_mode.rs](../../tests/test_sludge_mode.rs).


## Maelstrom Mode

`GameMode::Maelstrom` (menu entry "Maelstrom Mode") is a survival mode wrapped in a permanent rainstorm at sea (`weather` is forced to `Weather::Rain` every tick, and the random weather roll skips this mode):

- **Vortex:** a single whirlpool vortex (`maelstrom`, rendered as a blue `@` terminal / blue cell web) spawns on a random empty cell on reset (`spawn_maelstrom`, never on a snake, an obstacle, an entity, or the food, at least 6 cells (Manhattan) from the spawn, with a `safe_zone_margin + 2` margin) and reforms the same way if it is ever lost. The vortex cell is mirrored into `obstacles` (re-synced every tick), so it is lethal on entry and a hard constraint in the A* search and the `is_safe_final_p` verified-safe path checks — the autopilot routes around it natively.
- **Scheduled drifts:** on every `MAELSTROM_DRIFT_PERIOD` (8) tick the vortex slides one cell in a shuffled random direction and `maelstrom_drifts` is incremented (shown as `Drifts: <count>` in the status bar); the obstacle mirror moves with it. A drift only lands on a cell that passes `maelstrom_cell_free` (never onto a snake, the cell any snake is about to enter, or any entity/hazard).
- **Scheduled swirls:** on every `MAELSTROM_PERIOD` (6) tick `manage_maelstrom_swirl` fires a swirl counted in `maelstrom_swirls` (shown as `Swirls: <count>`): every snake (player, player 2, arena bots) within `MAELSTROM_RADIUS` (4, Manhattan) of the vortex is shoved exactly one cell tangentially around it — `maelstrom_swirl_steps` rotates the radial vector 90 degrees and tries the primary then the opposite tangential direction — via `move_to(dest, false)`, so a shove never grows the snake. The food, bonus food, poison food, and power-up are also dragged one cell towards the vortex. Snakes are swirled sequentially so earlier shoves are respected by later checks, and shoved snakes get a "Swirl!" floating text.
- **Fairness:** `maelstrom_swirl_destination` accepts only a destination that passes `maelstrom_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), the vortex itself (mirrored into `obstacles`), the lightning column, every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble), and the Battle Royale safe zone — so **a swirl can never directly kill**; when neither tangential direction is fair the snake simply stays put. The autopilot survives natively: every shove lands on a verified-safe cell and the bot re-plans its verified-safe route from the shoved position next tick.
- **Calming:** eating food sets `maelstrom_calm = MAELSTROM_CALM_TICKS` (6) ("Calmed!"); while calm is positive it counts down once per tick and no swirl fires at all (shown as `Calm: <ticks>` in the status bar).
- **Persistence:** `maelstrom`, `maelstrom_swirls`, `maelstrom_drifts`, and `maelstrom_calm` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_maelstrom_mode.rs](../../tests/test_maelstrom_mode.rs) and the save/load test in `src/game/tests.rs`.
## Decay Mode

`GameMode::Decay` (menu entry "Decay Mode", entry 136 in the main menu) is a survival mode played on a decaying floor; the weather is pinned to `Weather::Clear` (ambient weather re-rolls are skipped in this mode) so cracks and pits stay visible:

- **State:** `decay_cracks: HashMap<Point, u32>` maps each cracked cell to its remaining ticks (`DECAY_DELAY` = 36 when the crack appears), `decay_pits: HashSet<Point>` holds collapsed cells (always mirrored into `obstacles`), and `decay_collapses` / `decay_tremors` / `decay_repairs` count collapses, tremors, and food patches.
- **Board setup:** on reset `seed_initial_decay_cracks` places `clamp(width*height/100, 2, 6)` cracks on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, or the food.
- **Head cracking:** every tick `manage_decay` inserts `DECAY_DELAY` for the current head cell of every snake (player, player 2, bots) that is not already an obstacle, so a snake's trail cracks behind it.
- **Scheduled tremors:** every 6th tick `manage_decay` drops up to `DECAY_TREMOR_CRACKS` (2) fresh cracks via `place_decay_crack` (capped at 5% of the board, `decay_tremors += 1`, "Tremor! (N)"). Fairness: a new crack is never placed on a snake body, on the cell any snake is about to enter (`singularity_next_heads`), on any entity (food, bonus/poison food, power-ups, portals, mines, the black hole, the tornado vortex, goblin, bosses, companion, turrets, lasers, meteors, rocks, XP gems), on an obstacle, on any hazard tile set (lightning column, fire, acid, puddles, lava, inferno, blaze, seismic rubble, ice, belts, bumpers, tramp pads, phase gates, thorns, spikes, fault tiles), or on an existing crack/pit.
- **Collapse:** each tick every crack that no snake occupies and that no snake is about to enter counts down; at zero the cell becomes a pit (`decay_pits` + `obstacles`, `decay_collapses += 1`). Because occupied and next-head cells are deferred, **a pit can never form under a snake or on its very next step** — the floor is dangerous but never cheap. Entering an existing pit kills via the standard obstacle collision.
- **Renewal:** at tick 200 every pit is removed from `obstacles`/`decay_pits` and all cracks are sealed ("The floor is renewed!").
- **Patching by food:** eating food removes every crack and fills every pit within Manhattan distance 1 of the eaten cell ("Patched N!", counted in `decay_repairs`).
- **Autopilot:** in `is_safe_final_p` (mode-gated) a cracked cell is rejected when its remaining timer is within the arrival `steps` — the same per-step pattern as predicted meteors — and the A* expansion, `path_is_safe`, and the trap-avoidance flood fill all gate on `is_safe_final_p`; pits are already hard obstacles. Since a pit never forms on an occupied or next-head cell, a collapse can never kill the bot, which replans from its actual head each tick.
- **Rendering:** cracks are a dark-yellow `'` (terminal) / brown cell (web), pits a dark-grey `#` (terminal) / dark-grey cell (web); the status bar shows `Cracks: <count> | Pits: <count> | Collapses: <count> | Tremors: <count> | Repairs: <count>` (`decay_str`).
- **Persistence:** `decay_cracks` (serialized as `[key, value]` pairs via `map_as_pairs`), `decay_pits`, `decay_collapses`, `decay_tremors`, and `decay_repairs` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_decay_mode.rs](../../tests/test_decay_mode.rs).

## Erosion Mode

`GameMode::Erosion` (menu entry "Erosion Mode", entry 139 in the main menu) is a survival mode where the floor crumbles behind every snake:

- **Weakening:** every tick `manage_erosion` refreshes `erode_timers[head] = ERODE_TICKS` (24) for the player's, player 2's, and every bot's head cell, then counts down every weakened cell that is *not* occupied — `erode_occupied_cells` collects every snake's whole `body_map` plus `singularity_next_heads` (the cells snakes are about to enter this tick). A cell whose timer reaches zero crumbles: it is inserted into `erode_holes`, mirrored into `obstacles` (lethal on entry), and counted in `erode_crumbles`. Cells already blocked by a foreign obstacle are not re-tracked, so resettle/reinforce never removes another system's obstacle.
- **Fairness:** occupied and next-head timers freeze, so the floor never crumbles under a snake's body or on the cell any snake is about to enter — erosion can never kill anyone cheaply.
- **Tremors:** every `ERODE_TREMOR_PERIOD` (6) ticks a tremor opens one fresh hole on a random cell passing `erode_cell_fair` (in bounds within the safe-zone margin and free of snakes, entities, and hazard tiles via `singularity_cell_free`, plus the lightning column and every lethal tile set), capped at `ERODE_MAX_HOLE_PCT` (15%) of the board, counted in `erode_tremors` ("Tremor!"). Weather is pinned to `Weather::Clear` so the floor stays visible.
- **Resettle:** at tick 200 of every 250-tick cycle the ground heals — every hole is removed from `erode_holes` and `obstacles`, and `erode_timers` is cleared ("The ground resettles!").
- **Reinforcement:** eating food heals the four orthogonally adjacent cells of the eaten cell — holes leave `obstacles` and timers are dropped ("Reinforced N cell(s)!").
- **Autopilot:** holes are hard obstacles in the A* search and the verified-safe path checks, and `is_safe_final_p` additionally rejects any cell whose `erode_timers` value is `<=` its arrival `steps`, so the bot never plans onto collapsing floor.
- **Status bar:** `Holes: <count> | Crumbles: <count> | Tremors: <count>` (`erode_str`). Holes render as a dark-grey `x` and cells with at most `ERODE_WARN` (6) ticks left as a dark-yellow `.` on the terminal, dark-grey / dark-yellow cells on the web.
- **Persistence:** `erode_timers`, `erode_holes`, `erode_crumbles`, and `erode_tremors` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_erosion_mode.rs](../../tests/test_erosion_mode.rs).

## Induction Mode

`GameMode::Induction` (menu entry "Induction Mode", entry 140 in the main menu) is a survival mode played among two permanent magnet poles:

- **Magnet poles:** on reset `spawn_induction_poles` seeds exactly two poles (`induction_poles`) on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, a launch pad, a bumper, or the food. Poles are walkable fixtures (they are *not* mirrored into `obstacles`) and render as a red `N` / cyan `S` (terminal) and red / cyan cells (web), white while the shield is up.
- **Tug:** when a snake's final head cell is orthogonally aligned with a pole (same row or same column) within `INDUCTION_RANGE` (6) cells, `induction_tug` moves the head one extra cell along the alignment axis — toward the pole while `induction_attract` is true, away from it while false — and counts it in `induction_tugs` ("Tug!"). The tug is applied to the player, player 2, and every bot right after the other movement modifiers in `update`.
- **Fairness:** the tug destination must pass `induction_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column, every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble), and the Battle Royale safe zone. If the destination is not fair the tug fizzles, so **a pole can never directly kill**.
- **Scheduled polarity flip:** every `INDUCTION_PERIOD` (12) ticks `manage_induction` flips `induction_attract` and counts it in `induction_flips` ("Polarity flips! (Attract/Repel)").
- **Scheduled relocation:** every `INDUCTION_RELOC_PERIOD` (30) ticks both poles relocate to fresh fair cells (`induction_relocs += 1`, "The poles relocate! (N)") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, or within 4 cells (Manhattan) of any snake head. Poles that end up under a fresh obstacle (e.g. a meteor crash) are scrapped and topped back up at the start of each tick.
- **Shield:** eating food sets `induction_shield = INDUCTION_SHIELD_TICKS` (12) ("Shielded!"); while the timer is positive it counts down once per tick and no tugs happen at all.
- **Autopilot:** tug landings are always fair, so a tugged bot is never killed by the tug itself; the bot replans from its actual head every tick and rides the poles natively.
- **Status bar:** `Poles: <count> | Pull: <Attract/Repel> | Tugs: <count> | Flips: <count> | Relocs: <count>` plus `Shielded: <ticks>` while shielded (`induction_str`).
- **Persistence:** `induction_poles`, `induction_attract`, `induction_tugs`, `induction_flips`, `induction_relocs`, and `induction_shield` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_induction_mode.rs](../../tests/test_induction_mode.rs).
## Sinkhole Mode

`GameMode::Sinkhole` (menu entry "Sinkhole Mode", entry 141 in the main menu) is a survival mode played on a board that slowly fills with sinkholes:

- **Seeding:** on reset `spawn_sinkholes` places `SINKHOLE_SEED` (3) sinkhole pits (`sinkholes`) on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, a mine, a portal, the black hole, the tornado, another patch, or the food. Sinkholes are mirrored into `obstacles`, so entering one kills the snake, and render as a dark-grey `"` (terminal) / dark-grey cell (web).
- **Scheduled collapses:** every 6th tick `manage_sinkhole` fires a collapse (`sinkhole_collapses += 1`, "The ground collapses! (+N)"), converting up to `SINKHOLE_PER_COLLAPSE` (2) telegraphed cells into a sinkhole, capped at 10% of the board (`sinkhole_cap`).
- **Telegraph:** `SINKHOLE_TELEGRAPH_TICKS` (3) ticks before a collapse (tick % 6 == 3) the threatened cells bubble in `sinkhole_warnings` (yellow `,` terminal / yellow cells web) if they pass the fairness check.
- **Fairness:** `sinkhole_cell_blocked` never collapses on a snake's body, on the cell any snake is about to enter this tick (player/p2 use their current direction, bots their queued direction), or on any entity or hazard tile (including the quicksand bog, tar pits, dominoes, polarity poles, blink pads, and the crossfire target); the check is re-run at collapse time, so a snake that moved onto a bubbling cell is spared — a collapse can never swallow anyone cheaply.
- **Cool-down:** at tick 200 of every 250-tick cycle all pits heal away ("The ground heals!") and the warnings clear; the collapse counter keeps counting.
- **Food:** eating food shores up the pit adjacent to the eaten cell ("Shored up N sinkhole(s)!"), removing it from `sinkholes` and `obstacles`.
- **Autopilot:** pits live in `obstacles`, so they are hard constraints in the A* search, the verified-safe `is_safe_final_p` checks, and the trap-avoidance flood fill — the bot routes around the pit natively. Stale sinkhole records are pruned each tick if another effect cleared obstacles mid-tick (e.g. a Bomb), keeping the mirror exact.
- **Weather:** pinned to `Weather::Clear` every tick (ambient weather randomization is skipped in this mode) so the pits stay visible.
- **Status bar:** `Pits: <count> | Collapses: <count>` (`sinkhole_str`).
- **Persistence:** `sinkholes`, `sinkhole_warnings`, and `sinkhole_collapses` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_sinkhole_mode.rs](../../tests/test_sinkhole_mode.rs).


## Dipole Mode

`GameMode::Dipole` (menu entry "Dipole Mode", entry 142 in the main menu) is a survival mode played among a permanent set of magnetic dipoles:

- **Magnetic dipoles:** on reset `dipole_pole_count()` = `clamp(width*height/80, 3, 6)` dipoles (`dipole_poles: HashMap<Point, DipolePol>`) are seeded with a random dipole (`DipolePol::Attract` / `DipolePol::Repel`) on fair cells at least 5 cells (Manhattan) from every snake head — never on a snake, on the cell any snake is about to enter, on any entity, on an obstacle, on a hazard tile, on another dipole, or on the food. Poles are walkable (they are *not* mirrored into `obstacles`) and render as a green `A` (Attract) / red `R` (Repel) on the terminal and green / red cells on the web. Weather is pinned to `Weather::Clear` so the dipoles stay visible.
- **Shove:** when a snake's final head ends a tick orthogonally adjacent to a dipole, `dipole_shove` shoves the head one cell — toward the dipole for `Attract` (landing on the walkable dipole cell) or away from it for `Repel` — and counts it in `dipole_shoves` ("Shove!"). The shove is applied to the player, player 2, and every bot at the same points where `tramp_launch` is applied in `update`. With several adjacent dipoles the first in reading order whose shove is fair wins.
- **Fairness:** the shove destination must pass `dipole_cell_free` — in bounds within the safe-zone margin and free of snake bodies and the cells snakes are about to enter (`singularity_cell_free`), plus the lightning column and every lethal tile set (fire, acid, puddles, lava, inferno, blaze, seismic rubble) and the Battle Royale safe zone. If no adjacent dipole has a fair shove the head simply stays put, so **a dipole can never directly kill**.
- **Scheduled flip:** every `DIPOLE_FLIP_PERIOD` (6) ticks `manage_dipole` flips every dipole between Attract and Repel (`dipole_flips += 1`, "Dipole flip!").
- **Scheduled relocation:** every `DIPOLE_RELOC_PERIOD` (12) ticks one random dipole relocates to a fresh fair cell (`dipole_shifts += 1`, "A dipole shifts!") — never on a snake, on the cell any snake is about to enter, on any entity or hazard tile, on another dipole, or within 4 cells (Manhattan) of any snake head. At `DIPOLE_RESET_TICK` (200) of every 250-tick cycle the dipoles discharge and a fresh layout reseeds via `spawn_dipole_poles` ("The dipoles discharge!"). Dipoles that end up under a fresh obstacle (e.g. a meteor crash) are scrapped at the start of each tick.
- **Degauss:** eating food removes the dipoles orthogonally adjacent to the eaten cell ("Degaussed N dipole(s)!"), counted in `dipole_shifts`.
- **Autopilot:** shove destinations are always fair, so a shoved bot is never killed by the shove itself; the bot replans from its actual head every tick and rides the dipoles natively.
- **Status bar:** `Poles: <count> | Shoves: <count> | Flips: <count> | Shifts: <count>` (`dipole_str`).
- **Persistence:** `dipole_poles`, `dipole_shoves`, `dipole_flips`, and `dipole_shifts` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
- **Tests:** see [tests/test_dipole_mode.rs](../../tests/test_dipole_mode.rs) and the save/load unit test `test_save_and_load_dipole_mode` in `src/game/tests.rs`.

## Blight Mode

`GameMode::Blight` (menu entry "Blight Mode", entry 143 in the main menu) is a survival hazard mode where patches of blight slowly spread across the board. Unlike the Quicksand bog (walkable but sticky) and the Mire pools, blight is a **hard wall**:

- **Blight tiles:** `blight_tiles` is a `HashSet<Point>` mirrored into `obstacles`, so every blight cell blocks movement like a wall and is lethal on entry (magenta `▓` on the terminal / medium-purple `#9370DB` cell on the web).
- **Seeding:** on reset `spawn_blight_patches` places `BLIGHT_SEED_PATCHES` (3) patches of up to `BLIGHT_PATCH_SIZE` (3) cells each on fair cells at least 5 cells (Manhattan) from the spawn — never on a snake, an entity, an obstacle, or the food.
- **Scheduled spread:** every `BLIGHT_SPREAD_PERIOD` (6th) tick `manage_blight` → `blight_spread` lets each patch creep onto one random fair adjacent cell (60% chance per patch) and occasionally spawns a brand-new patch, all capped at `width*height/BLIGHT_MAX_DIVISOR` (20% of the board). Spreads are counted in `blight_spreads` ("The blight spreads! (+N)").
- **Fairness:** `blight_cell_fair` rejects any cell outside the safe-zone margin, on an obstacle (which includes existing blight), on any snake body, on the cell any snake is about to enter this tick (`singularity_next_heads`), or on any entity/hazard tile — so the blight can never spawn on or under a snake and never cheaply kill.
- **Dry-up:** at tick 200 of every 250-tick cycle all blight withers — removed from both `blight_tiles` and `obstacles` ("The blight withers!").
- **Draining:** eating food calls `blight_cleanse_at(final_head)`, clearing the four orthogonally adjacent blight cells (`blight_clears += N`, "Cleansed N blight!").
- **Weather:** pinned to `Weather::Clear` every tick (and the ambient weather re-roll is skipped) so the blight is always visible.
- **Autopilot:** because blight lives in `obstacles`, the A* search and the verified-safe `is_safe_final_p` checks treat it as a hard obstacle, so the bot re-plans around the spreading blight each tick and survives natively.
- **Status bar:** `Blight: <count> | Spreads: <count> | Clears: <count>` (`blight_hud`).
- **Persistence:** `blight_tiles`, `blight_spreads`, and `blight_clears` are stored in `SaveState` and `HistoryState` (save/load and time-rewind).
- **Tests:** see [tests/test_blight_mode.rs](../../tests/test_blight_mode.rs).

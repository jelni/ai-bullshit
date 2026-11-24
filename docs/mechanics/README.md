# Game Mechanics

The game features several complex mechanics beyond simple snake movement:

- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time). Verification also predicts rival snakes: each rival is assumed to keep moving straight along its current direction (through portals and wrapping) for up to `Game::RIVAL_HEAD_HORIZON` (8) steps, and any path cell that a rival's predicted head occupies at that cell's arrival step is rejected as a head-on collision; the A* expansion avoids these predicted lanes as well (see [tests/test_bot_rival_prediction.rs](../../tests/test_bot_rival_prediction.rs)). On top of that, `path_is_safe` simulates every other snake forward along its committed direction queue for up to 24 steps (`predicted_enemy_occupancy`) and treats any cell predicted to be occupied by an enemy head or body at a given step as a hard constraint, so queued turns and swept body lanes are avoided too. Paths are also checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists. In multiplayer modes the autopilot also avoids contested targets (`contested_targets`): any food, power-up, or objective that a rival snake is predicted to reach strictly first (by wrap/portal-aware travel distance via `travel_dist`, or because the rival's committed head path crosses it sooner) is skipped in favor of an uncontested target, and only raced for when nothing else verifies (`astar_search_verified_contested`; see [tests/test_bot_contested_targets.rs](../../tests/test_bot_contested_targets.rs)).
- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time). Verification also predicts rival snakes: each rival is assumed to keep moving straight along its current direction (through portals and wrapping) for up to `Game::RIVAL_HEAD_HORIZON` (8) steps, and any path cell that a rival's predicted head occupies at that cell's arrival step is rejected as a head-on collision; the A* expansion avoids these predicted lanes as well (see [tests/test_bot_rival_prediction.rs](../../tests/test_bot_rival_prediction.rs)). On top of that, `path_is_safe` simulates every other snake forward along its committed direction queue for up to 24 steps (`predicted_enemy_occupancy`) and treats any cell predicted to be occupied by an enemy head or body at a given step as a hard constraint, so queued turns and swept body lanes are avoided too. Paths are also checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists.
- **Hunter AI (rival cut-off):** in modes with rival snakes (PlayerVsBot, BotVsBot, MassiveMultiplayer, Tron, King of the Hill, Capture the Flag, Zombie), a bot that is strictly longer than a nearby rival switches from pure survival to hunting: it projects each shorter rival's head up to `Game::CUTOFF_HORIZON` (12) steps ahead along its committed direction queue (`cutoff_targets`) and, if it can reach the cell where the rival's head will be after `k` steps within `k - 1` steps, it moves there so the rival crashes into its body on step `k` (collision checks use pre-move bodies, so the mover's body after `k - 1` moves is what kills). Kill paths must pass the same verified-safe, non-trapping checks as any other path, plus `path_cuts_off_rival`, which simulates the path against every shorter rival's predicted head path (the mover's own tail cell does not count as a kill, since it vacates in time). Among verified kill paths the shortest one is chosen; when no safe cut-off exists the bot falls back to its normal target selection. A CTF flag carrier never hunts. This applies to the player bot, the Player 2 bot, and the arena bots (see [tests/test_bot_cutoff_hunting.rs](../../tests/test_bot_cutoff_hunting.rs)).
- **Flow-Field Autopilot (Massive Multiplayer / Zombie):** Arena bots in these modes normally follow a BFS flow field toward their targets for performance. Before a flow-field step is taken, the implied route is extracted with `follow_flow_field` and held to the same standards as the A* autopilot: every cell must pass `path_is_safe`, and `path_keeps_tail_reachable` must confirm the bot's tail stays reachable after simulating the route. If the flow-field route is unsafe or trapping, the bot falls back to the verified A* search, then tail-chasing, then the flood-fill survival move.
- **Power-ups:** Players can collect various power-ups that modify gameplay (e.g., speed boosts, invincibility, or score multipliers).
- **Daily Challenges:** Deterministic challenges generated using a specific global seed.
- **Missions/Bounties:** Dynamic quests spawned during the game.

## Weather System

The `weather` field on the game state cycles through `Clear`, `Rain`, `Snow`, `Storm`, `Tornado`, `Sandstorm`, `Earthquake`, `Eclipse` and `Fog` (randomly over time, and forced per-mode/per-planet in some cases). Each weather type has concrete gameplay effects, not just visuals:

- **Snow:** occasionally delays the autopilot bot (except in Blizzard mode, where the storm is permanent but the bot is never delayed).
- **Sandstorm:** occasionally shuffles the food to a neighboring cell.
- **Earthquake:** occasionally adds or removes an obstacle. In Quake Mode the earthquake is permanent and a quake is guaranteed every 6th tick. In Earthquake Mode the earthquake is also permanent, with a fair scheduled tremor every 5th tick.
- **Tornado:** occasionally relocates the food. In Tornado Mode the twister is permanent: a roaming vortex drifts every 6th tick and drags items towards itself, and every 8th tick a board-wide gust shoves every snake one cell in a shared random direction (see below).
- **Storm:** occasionally calls down a lightning column that devastates a whole row (including bosses standing in it). In Storm Mode the storm is permanent and a strike is guaranteed every 6th tick.
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
- **Scheduled showers:** on every 12th tick (`tick_counter.is_multiple_of(12)`) a meteor shower fires: a burst of up to 3 meteors spawns in distinct columns at the top row. Each shower increments `meteor_showers`.
- **Fairness:** in Asteroids mode, meteor spawn columns (ambient and shower alike) are filtered by `meteor_safe_spawn_columns`, which never returns the column a snake (player, player 2, or arena bot) currently occupies or is about to enter this tick, nor a column that already has a meteor in flight — the same fairness rule as Storm Mode's lightning strikes. Other modes keep the old fully random spawn.
- **Deflection:** eating food deflects the most imminent meteor (the lowest one in flight) with a "Deflected!" flash, giving the player a way to neutralize the closest incoming impact.
- **Impact:** a falling meteor destroys obstacles and mines in its path, despawns at the bottom edge, and kills any snake whose head or body it lands on.
- **Autopilot:** a meteor's fall lane is a hard hazard in the A* heuristic/edge costs and in `is_safe_final_p` (a path cell in the meteor's column at or below the meteor is rejected when the meteor would reach it within the cell's true arrival step), and `path_is_safe` re-checks it per step, so the verified-safe autopilot dodges incoming meteors natively.
- **Persistence:** the `meteor_showers` counter is stored in `SaveState` and `HistoryState` (save/load and time-rewind stay consistent), and `meteors` were already persisted. The status bar shows a `Showers: <count>` counter in this mode.
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
- **Board-wide gusts:** on every 8th tick `manage_tornado_gust` fires a gust in one shared random direction that shoves every snake (player, player 2, arena bots) exactly one cell via `gust_destination`, and drags the food one cell with it. A shove uses `move_to(dest, false)`, so it never grows the snake. The gust never shoves out of bounds (unless the board wraps, in which case the destination wraps like a normal move), into an obstacle, into any snake's body, onto the vortex, or into anything lethal or occupied (mines, black hole, active lightning column, bosses, lasers, meteors, turrets); snakes are shoved sequentially so earlier shoves are respected by later checks, and a blocked destination simply fizzles. Shoved snakes get a "Gust!" floating text. The autopilot survives natively: every shove lands on a verified-safe cell and the bot re-plans its verified-safe route from the shoved position next tick.
- **Fairness:** a gust never lands on a snake's body, on the cell any snake is about to enter this tick, or on any entity (food, bonus/poison food, power-ups, portals, mines, black hole, goblin, bosses, companion, turrets, lasers, meteors, xp gems, equipment boxes, crops, eggs, resources, the KoTH zone, flags), so the twister is dangerous but never cheap.
- **Destruction:** when the vortex moves, all obstacles and mines on and around its new cell (Manhattan distance <= 1) are sucked in and removed.
- **Item pull:** every tick the vortex drags the food, bonus food, poison food, and power-up one cell towards itself (20% chance each, reusing the black-hole pull) if the destination is free.
- **Lethality:** a snake (player, player 2, or arena bot) whose head enters the vortex cell dies, unless invincible.
- **Autopilot:** the vortex cell is a hard hazard in `is_safe_final_p` (so verified-safe path checks reject it at every step), carries a large heuristic penalty and edge cost in both `astar_search` and `get_boss_path`, and is a blocked cell in `trap_blocked_cells`, so bots and bosses route around the twister natively.
- **Persistence:** `tornado` and `tornado_gusts` are stored in `SaveState` and `HistoryState` (so save/load and time-rewind stay consistent).
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

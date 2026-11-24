# Game Mechanics

The game features several complex mechanics beyond simple snake movement:

- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time) and checked for trap avoidance (after simulating the path, a flood fill must confirm the snake's tail stays reachable). Among all verified-safe, non-trapping paths to the current targets, `astar_search_verified` picks the most survivable one by scoring the reachable free space after arrival (`reachable_space_after_path`) minus the path length, so the bot proactively prefers roomy routes over cramped ones while still favoring shorter paths between equally roomy options. Tail-chasing remains the survival fallback when no verified-safe, non-trapping path to any target exists.
- **Power-ups:** Players can collect various power-ups that modify gameplay (e.g., speed boosts, invincibility, or score multipliers).
- **Daily Challenges:** Deterministic challenges generated using a specific global seed.
- **Missions/Bounties:** Dynamic quests spawned during the game.

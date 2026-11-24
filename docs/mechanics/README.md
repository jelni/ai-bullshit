# Game Mechanics

The game features several complex mechanics beyond simple snake movement:

- **Autopilot (Bot Mode):** An implementation of the A* search algorithm is used to allow the snake to navigate autonomously, avoiding obstacles, enemies (like bosses and lasers), and targeting food. Candidate paths are verified step-by-step (`path_is_safe` re-checks `is_safe_final_p` at each cell's true arrival time); unsafe paths are rejected in favor of safer targets, with tail-chasing as the survival fallback when no verified-safe path to any target exists.
- **Power-ups:** Players can collect various power-ups that modify gameplay (e.g., speed boosts, invincibility, or score multipliers).
- **Daily Challenges:** Deterministic challenges generated using a specific global seed.
- **Missions/Bounties:** Dynamic quests spawned during the game.

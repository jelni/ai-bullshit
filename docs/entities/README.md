# Entities

The primary entities in the game include:

- **Snake:** (`src/snake.rs`) Managed as a sequence of `Point`s. Handles logic for movement, growth, and collision with itself.
- **Bosses:** Adversaries that introduce complex pathing requirements to evade.
- **Lasers/Mines:** Static and dynamic obstacles.
- **Meteors:** Environmental hazards that spawn over time.
- **Food/Crops/Artifacts:** Items the snake collects to progress.

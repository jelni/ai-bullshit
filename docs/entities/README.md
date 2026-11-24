# Entities

The primary entities in the game include:

- **Snake:** (`src/snake.rs`) Managed as a sequence of `Point`s. Handles logic for movement, growth, and collision with itself.
- **Bosses:** Adversaries that introduce complex pathing requirements to evade.
- **Lasers/Mines:** Static and dynamic obstacles.
- **Meteors:** Environmental hazards that spawn over time.
- **Lava/Vents:** In Volcano mode, vents seed a spreading lava field (`lava_tiles`, mirrored into `obstacles`) that is lethal on entry and treated as a hard obstacle by the autopilot.
- **Food/Crops/Artifacts:** Items the snake collects to progress.

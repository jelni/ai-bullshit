# Entities

The primary entities in the game include:

- **Snake:** (`src/snake.rs`) Managed as a sequence of `Point`s. Handles logic for movement, growth, and collision with itself.
- **Bosses:** Adversaries that introduce complex pathing requirements to evade.
- **Lasers/Mines:** Static and dynamic obstacles.
- **Meteors:** Environmental hazards that spawn over time (frequently in Asteroids and Meteor Shower modes). A meteor enters at the top of a column and falls one cell every 2 ticks, smashing obstacles/mines and killing any snake it touches; scheduled Meteor Shower strikes always pick a column no snake occupies or is about to enter. Shower meteors in Asteroids mode can also drift sideways (`Meteor::vx`) and bounce off the side walls; the autopilot predicts their exact trajectory (`meteor_position_after` / `meteor_would_hit`).
- **Lava/Vents:** In Volcano mode, vents seed a spreading lava field (`lava_tiles`, mirrored into `obstacles`) that is lethal on entry and treated as a hard obstacle by the autopilot.
- **Food/Crops/Artifacts:** Items the snake collects to progress.

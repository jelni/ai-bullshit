import re

# Fix game.rs match error
with open('src/game.rs', 'r') as f:
    content = f.read()

content = content.replace("    NftDrop,\n                    _ => PowerUpType::ExtraLife,", "                    7 => PowerUpType::NftDrop,\n                    _ => PowerUpType::ExtraLife,")
content = content.replace("self.rng.gen_range(0..8,)", "self.rng.gen_range(0..9,)")

with open('src/game.rs', 'w') as f:
    f.write(content)

# Fix main.rs tick rate match error
with open('src/main.rs', 'r') as f:
    main_content = f.read()

main_content = main_content.replace(
    "| game::PowerUpType::ClearObstacles\n                    | game::PowerUpType::ScoreMultiplier => {},",
    "| game::PowerUpType::ClearObstacles\n                    | game::PowerUpType::ScoreMultiplier\n                    | game::PowerUpType::NftDrop => {},"
)

with open('src/main.rs', 'w') as f:
    f.write(main_content)

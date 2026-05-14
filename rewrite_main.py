import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Fix the duplicate 14 and insert 13
content = content.replace("            14 => {\n                game.mode = game::GameMode::BossRush;\n                game.reset();\n            },\n            14 => {\n                let _ = game.load_game();\n            },", "            13 => {\n                game.mode = game::GameMode::BossRush;\n                game.reset();\n            },\n            14 => {\n                let _ = game.load_game();\n            },")

with open('src/main.rs', 'w') as f:
    f.write(content)

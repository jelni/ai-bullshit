import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Match the switch statement
content = re.sub(r'(\s*12\s*=>\s*\{\n\s*game\.mode\s*=\s*game::GameMode::Speedrun;\n\s*game\.reset\(\);\n\s*\},)',
                 r'\1\n            13 => {\n                game.mode = game::GameMode::BossRush;\n                game.reset();\n            },', content)

# Offset all subsequent matches
for i in range(21, 12, -1):
    content = re.sub(rf'(\s*){i}(\s*=>)', rf'\g<1>{i+1}\g<2>', content)

# Change index limits
content = re.sub(r'game\.menu_selection\s*=\s*21', r'game.menu_selection = 22', content)
content = re.sub(r'game\.menu_selection\s*<\s*21', r'game.menu_selection < 22', content)

with open('src/main.rs', 'w') as f:
    f.write(content)

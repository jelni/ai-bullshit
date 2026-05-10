import re

with open('src/game.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'(\n    #\[serde\(skip\)\]\n    pub bot_path: std::collections::VecDeque<Point>,\n)',
    r'\n    pub bot_path: std::collections::VecDeque<Point>,\n',
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

with open('src/game.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'(pub auto_pilot: bool,\n)',
    r'\1    #[serde(skip)]\n',
    content
)

# wait, we need to modify SaveState, not Game.
# Let's read where SaveState is defined.

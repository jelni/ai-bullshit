import re

with open('src/game.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'(pub auto_pilot: bool,\n\s*)(pub bot_path: std::collections::VecDeque<Point>,\n)',
    r'\1#[serde(skip)]\n    \2',
    content
)

content = re.sub(
    r'(auto_pilot: self\.auto_pilot,\n\s*\})',
    r'\1',
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

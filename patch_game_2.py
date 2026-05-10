import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# 1. Update Game struct
content = re.sub(
    r'(pub previous_state: Option<GameState,>,\n\s*pub auto_pilot: bool,\n)\}',
    r'\1    pub bot_path: std::collections::VecDeque<Point>,\n}',
    content
)

# 2. Update Game::new initialization
content = re.sub(
    r'(previous_state: None,\n\s*auto_pilot: false,\n\s*)\}',
    r'\1    bot_path: std::collections::VecDeque::new(),\n        }',
    content
)

# 3. Update load_game_from_file (no need to save/load bot_path, just init empty)
content = re.sub(
    r'(self\.auto_pilot = state\.auto_pilot;\n\s*self\.state = GameState::Paused;)',
    r'\1\n                self.bot_path.clear();',
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

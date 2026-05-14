import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# 1. Add BossRush to GameMode enum
content = re.sub(r'(CustomLevel,\n\s*Speedrun,)\n}', r'\1\n    BossRush,\n}', content)

# 2. Add to reset() match
content = re.sub(r'(\|\s*GameMode::Speedrun\n\s*\|\s*GameMode::Survival\n\s*\|\s*GameMode::Zen\n\s*\|\s*GameMode::Maze\n\s*\|\s*GameMode::Cave\n\s*\|\s*GameMode::CustomLevel\s*=>\s*\{)',
                 r'| GameMode::Speedrun\n            | GameMode::BossRush\n            | GameMode::Survival\n            | GameMode::Zen\n            | GameMode::Maze\n            | GameMode::Cave\n            | GameMode::CustomLevel => {', content)

# 3. Add to avoid
content = re.sub(r'(\|\|\s*self\.mode\s*==\s*GameMode::Speedrun\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Survival\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Zen\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Maze\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Cave\n\s*\|\|\s*self\.mode\s*==\s*GameMode::CustomLevel)',
                 r'|| self.mode == GameMode::Speedrun\n                || self.mode == GameMode::BossRush\n                || self.mode == GameMode::Survival\n                || self.mode == GameMode::Zen\n                || self.mode == GameMode::Maze\n                || self.mode == GameMode::Cave\n                || self.mode == GameMode::CustomLevel', content)

# 4. Add to ref_snake
content = re.sub(r'(\|\|\s*self\.mode\s*==\s*GameMode::Speedrun\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Survival\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Zen\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Maze\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Cave\n\s*\|\|\s*self\.mode\s*==\s*GameMode::CustomLevel)',
                 r'|| self.mode == GameMode::Speedrun\n            || self.mode == GameMode::BossRush\n            || self.mode == GameMode::Survival\n            || self.mode == GameMode::Zen\n            || self.mode == GameMode::Maze\n            || self.mode == GameMode::Cave\n            || self.mode == GameMode::CustomLevel', content)

# 5. Add to p1_dead handler
content = re.sub(r'(\|\|\s*self\.mode\s*==\s*GameMode::Speedrun\n\s*\|\|\s*self\.mode\s*==\s*GameMode::Survival\n\s*\{\n\s*self\.handle_death\("You Died!"\);)',
                 r'|| self.mode == GameMode::Speedrun\n                || self.mode == GameMode::BossRush\n                || self.mode == GameMode::Survival\n            {\n                self.handle_death("You Died!");', content)

with open('src/game.rs', 'w') as f:
    f.write(content)

import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# 2. Modify tick() logic to spawn boss in BossRush mode if none exists
boss_logic = """                self.chat_log.push_back((
                    web_time::Instant::now(),
                    "BOSS INCOMING!".to_string(),
                    Color::Red,
                ));"""
boss_logic_new = """                self.chat_log.push_back((
                    "System: BOSS INCOMING!".to_string(),
                    crate::color::Color::Red,
                ));"""

content = content.replace(boss_logic, boss_logic_new)

with open('src/game.rs', 'w') as f:
    f.write(content)

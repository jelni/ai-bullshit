import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# 1. Update PowerUpType enum
content = re.sub(
    r'(ScoreMultiplier,\n)',
    r'\1    NftDrop,\n',
    content
)

# 2. Update powerup collision logic
collision_logic = """                self.snake.shrink_tail();
            } else if p.p_type == PowerUpType::ClearObstacles {
                self.obstacles.clear();
            } else if p.p_type == PowerUpType::NftDrop {
                self.score += 50;
                self.skin = '$';
            } else {
                p.activation_time = Some(SystemTime::now(),);
            }"""

content = re.sub(
    r'                self\.snake\.shrink_tail\(\);\n            \} else if p\.p_type == PowerUpType::ClearObstacles \{\n                self\.obstacles\.clear\(\);\n            \} else \{\n                p\.activation_time = Some\(SystemTime::now\(\),\);\n            \}',
    collision_logic,
    content
)

# 3. Update powerup instant removal logic
instant_logic = """            && (p.p_type == PowerUpType::ExtraLife
                || p.p_type == PowerUpType::Shrink
                || p.p_type == PowerUpType::ClearObstacles
                || p.p_type == PowerUpType::NftDrop)
            && p.activation_time.is_none()"""

content = re.sub(
    r'            && \(p\.p_type == PowerUpType::ExtraLife\n                \|\| p\.p_type == PowerUpType::Shrink\n                \|\| p\.p_type == PowerUpType::ClearObstacles\)\n            && p\.activation_time\.is_none\(\)',
    instant_logic,
    content
)

# 4. Update manage_power_ups to spawn NftDrop
spawn_logic = """                let p_type = match self.rng.gen_range(0..9,) {
                    0 => PowerUpType::SlowDown,
                    1 => PowerUpType::SpeedBoost,
                    2 => PowerUpType::Invincibility,
                    3 => PowerUpType::PassThroughWalls,
                    4 => PowerUpType::Shrink,
                    5 => PowerUpType::ClearObstacles,
                    6 => PowerUpType::ScoreMultiplier,
                    7 => PowerUpType::NftDrop,
                    _ => PowerUpType::ExtraLife,
                };"""

content = re.sub(
    r'                let p_type = match self\.rng\.gen_range\(0\.\.8,\) \{\n                    0 => PowerUpType::SlowDown,\n                    1 => PowerUpType::SpeedBoost,\n                    2 => PowerUpType::Invincibility,\n                    3 => PowerUpType::PassThroughWalls,\n                    4 => PowerUpType::Shrink,\n                    5 => PowerUpType::ClearObstacles,\n                    6 => PowerUpType::ScoreMultiplier,\n                    _ => PowerUpType::ExtraLife,\n                \};',
    spawn_logic,
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

# 5. Update UI rendering for NftDrop
with open('src/ui.rs', 'r') as f:
    ui_content = f.read()

ui_logic = """        } else if power_up.p_type == crate::game::PowerUpType::ScoreMultiplier {
            stdout.queue(SetForegroundColor(Color::Green,))?;
            write!(stdout, "$")?;
        } else if power_up.p_type == crate::game::PowerUpType::NftDrop {
            stdout.queue(SetForegroundColor(Color::Yellow,))?;
            write!(stdout, "N")?;
        } else {"""

ui_content = re.sub(
    r'        \} else if power_up\.p_type == crate::game::PowerUpType::ScoreMultiplier \{\n            stdout\.queue\(SetForegroundColor\(Color::Green,\),\)\?;\n            write!\(stdout, "\$"\)\?;\n        \} else \{',
    ui_logic,
    ui_content
)

# Update UI power_up_name match. We don't really need to add it here because NftDrop is instant and shouldn't have a timer, but just in case, let's look for match power_up.p_type in src/main.rs or src/ui.rs
# It's in src/ui.rs
powerup_name_logic = """                crate::game::PowerUpType::ScoreMultiplier => "2x Score",
                crate::game::PowerUpType::NftDrop => "NFT Drop",
            };"""

ui_content = re.sub(
    r'                crate::game::PowerUpType::ScoreMultiplier => "2x Score",\n            \};',
    powerup_name_logic,
    ui_content
)


with open('src/ui.rs', 'w') as f:
    f.write(ui_content)

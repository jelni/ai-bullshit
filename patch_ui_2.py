import re

with open('src/ui.rs', 'r') as f:
    content = f.read()

bot_path_logic = """    // Draw bot path
    if game.auto_pilot {
        stdout.queue(SetForegroundColor(Color::DarkGrey,))?;
        for p in &game.bot_path {
            // Do not overwrite food or bonus food with path dot
            let is_food = *p == game.food;
            let is_bonus_food = game.bonus_food.map_or(false, |(bp, _)| bp == *p);
            if !is_food && !is_bonus_food {
                stdout.queue(cursor::MoveTo(p.x, p.y,))?;
                write!(stdout, "·")?;
            }
        }
    }"""

content = re.sub(
    r'    // Draw bot path\n    if game\.auto_pilot \{\n        stdout\.queue\(SetForegroundColor\(Color::DarkGrey,\),\)\?;\n        for p in &game\.bot_path \{\n            stdout\.queue\(cursor::MoveTo\(p\.x, p\.y,\),\)\?;\n            write!\(stdout, "·"\)\?;\n        \}\n    \}',
    bot_path_logic,
    content
)

with open('src/ui.rs', 'w') as f:
    f.write(content)

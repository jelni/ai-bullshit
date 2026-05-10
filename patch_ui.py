import re

with open('src/ui.rs', 'r') as f:
    content = f.read()

bot_path_code = """
    // Draw bot path
    if game.auto_pilot {
        stdout.queue(SetForegroundColor(Color::DarkGrey,))?;
        for p in &game.bot_path {
            stdout.queue(cursor::MoveTo(p.x, p.y,))?;
            write!(stdout, "·")?;
        }
    }

    // Draw snake"""

content = content.replace('    // Draw snake', bot_path_code)

with open('src/ui.rs', 'w') as f:
    f.write(content)

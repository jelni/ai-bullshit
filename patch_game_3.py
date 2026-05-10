import re

with open('src/game.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'(if self\.auto_pilot\n\s*&& self\.snake\.direction_queue\.is_empty\(\)\n\s*&& let Some\(dir,\) = self\.calculate_autopilot_move\(\)\n\s*\{\n\s*self\.snake\.direction_queue\.push_back\(dir,\);\n\s*\})',
    r'''if self.auto_pilot
            && self.snake.direction_queue.is_empty()
            && let Some((dir, path)) = self.calculate_autopilot_path()
        {
            self.snake.direction_queue.push_back(dir,);
            self.bot_path = path;
        } else if !self.auto_pilot {
            self.bot_path.clear();
        }''',
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

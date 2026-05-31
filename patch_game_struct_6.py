import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Let's fix get_boss_path target check
# The target is likely unreachable if it's the snake head itself because self.snake.body_map.contains_key(&next_p) is true.
# Ah! A* tries to reach `target`. But `target` is the snake head!
# `self.snake.body_map.contains_key(&next_p)` makes `target` an invalid move!
# So A* returns None because the target is occupied by the snake head!
# Let's fix `get_boss_path` to allow moving onto the target.

search_str = """                    let mut can_move = true;
                    if self.snake.body_map.contains_key(&next_p) {
                        can_move = false;
                    } else if self.obstacles.contains(&next_p) {"""

replace_str = """                    let mut can_move = true;
                    if next_p != target && self.snake.body_map.contains_key(&next_p) {
                        can_move = false;
                    } else if self.obstacles.contains(&next_p) {"""

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Fixed target collision!")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# I will update `get_boss_path` to use a Reverse wrapper for BinaryHeap because AStarState might not be Min-Heap by default.
# Let's check `AStarState` implementation first.

search_str = """                        let dir_opt = if let Some(next_p) = self.get_boss_path(boss.position, target_pos, boss.kind) {"""

replace_str = """                        let dir_opt = if let Some(next_p) = self.get_boss_path(boss.position, target_pos, boss.kind).or_else(|| self.bfs_pathfind(boss.position, target_pos).map(|d| Self::calculate_next_head_dir(boss.position, d))) {"""

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Added fallback!")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Since I solved clippy error for update_tick by removing the unfulfilled expectation
# I now need to address the other review comment:
# "Performance Regression (Blocking): The new A* algorithm is completely unbounded, unlike the previous astar_search(..., 3). If a path is unreachable, it will blindly search the entire board. Because it evaluates every single frame/tick, this will cause a catastrophic performance death spiral if a boss cannot reach the player."
# And:
# "Logic Bug in Pathfinding Bounds (Blocking): The bounds check nx > i32::from(margin) (where margin is at least 1) means nx must be 2 or greater. It completely ignores the outermost playable tiles (e.g., x=1 and x=width-2). If the player hugs the wall, the boss's A* will fail to find a path"

# Let's fix get_boss_path to include an iteration limit and fix the bounds.

search_str = """                if nx > i32::from(margin) && nx < i32::from(self.width) - 1 - i32::from(margin) &&
                   ny > i32::from(margin) && ny < i32::from(self.height) - 1 - i32::from(margin) {"""

replace_str = """                if nx >= i32::from(margin) && nx < i32::from(self.width) - i32::from(margin) &&
                   ny >= i32::from(margin) && ny < i32::from(self.height) - i32::from(margin) {"""

content = content.replace(search_str, replace_str)

search_limit = """        while let Some(AStarState { position: current, .. }) = open_set.pop() {"""

replace_limit = """        let mut iterations = 0;
        while let Some(AStarState { position: current, .. }) = open_set.pop() {
            iterations += 1;
            if iterations > 300 {
                break;
            }"""

content = content.replace(search_limit, replace_limit)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

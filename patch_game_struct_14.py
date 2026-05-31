import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# I see "warning: this lint expectation is unfulfilled" at line 2682.
# Let's remove it if it's there.
# Oh, my previous replace was looking for the exact string, but it had `clippy::too_many_lines` on line 2682.

search_str = """    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    pub fn get_boss_path"""

replace_str = "    pub fn get_boss_path"

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Removed from get_boss_path!")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

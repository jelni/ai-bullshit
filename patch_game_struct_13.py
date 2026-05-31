import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# I see a warning: "this lint expectation is unfulfilled" at line 2682.
# That means `update_tick` no longer needs the `too_many_lines` exception because it's broken up or refactored.
# Wait! I added it back on `get_boss_path` before, so `update_tick` probably had it and now doesn't need it?
# Let's completely remove the #[expect] on update_tick since it's unfulfilled!

search_str = """    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn update_tick(&mut self) {"""

replace_str = "    fn update_tick(&mut self) {"

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Removed unfulfilled lint expectation on update_tick!")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

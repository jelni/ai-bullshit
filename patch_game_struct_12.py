import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# I want to add #[expect(clippy::too_many_lines, reason = "Game loop inherently requires handling multiple states and events")]
# back to update_tick. It was removed in patch_game_struct_7.py.

search_str = "    fn update_tick(&mut self) {"
replace_str = """    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn update_tick(&mut self) {"""

# Because I might have multiple `fn update_tick` occurences (or one inside the struct), let's just replace the FIRST ONE that matches exactly the signature.
if "fn update_tick(&mut self) {" in content:
    content = content.replace("    fn update_tick(&mut self) {", replace_str, 1)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

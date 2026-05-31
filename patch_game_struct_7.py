import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Fix unfulfilled lint expectation for clippy::too_many_lines on update_tick which is now smaller or changed
search_str = """    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn update_tick(&mut self) {"""

replace_str = "    fn update_tick(&mut self) {"

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Fixed clippy warning on update_tick!")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

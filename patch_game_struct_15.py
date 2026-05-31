import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Let's use regex to find and remove the #[expect] that is right above pub fn get_boss_path
content = re.sub(
    r"    #\[expect\([^\]]+\)\]\s+pub fn get_boss_path",
    r"    pub fn get_boss_path",
    content,
    flags=re.MULTILINE | re.DOTALL
)

# And put it back on update_tick
content = re.sub(
    r"    fn update_tick\(&mut self\) \{",
    r"""    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn update_tick(&mut self) {""",
    content,
    flags=re.MULTILINE | re.DOTALL
)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

import re

with open('src/game.rs', 'r') as f:
    content = f.read()

content = content.replace("    pub fn generate_cave_obstacles(", """    /// # Panics
    ///
    /// Panics if coordinate conversion to `usize` fails during cave generation.
    pub fn generate_cave_obstacles(""")

with open('src/game.rs', 'w') as f:
    f.write(content)

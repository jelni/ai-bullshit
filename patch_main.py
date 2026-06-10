import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Fix the array indices
for i in range(70, 37, -1):
    content = content.replace(f"            {i} => ", f"            {i+1} => ")

# Fix limits in menu handle func
content = content.replace("game.menu_selection = 70;", "game.menu_selection = 71;")
content = content.replace("game.menu_selection < 70 {", "game.menu_selection < 71 {")

# Add the new mode
content = content.replace(
    "            38 => {",
    "            38 => {\n                game.mode = game::GameMode::Dodgeball;\n                game.reset();\n            },\n            39 => {"
)


with open("src/main.rs", "w") as f:
    f.write(content)

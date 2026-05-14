import re

with open('src/game.rs', 'r') as f:
    content = f.read()

content = content.replace("game.tick();", "game.update();")

with open('src/game.rs', 'w') as f:
    f.write(content)

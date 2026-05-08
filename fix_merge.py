import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# Replace all conflict blocks with HEAD version
content = re.sub(r'<<<<<<< HEAD\n(.*?)\n=======\n.*?\n>>>>>>> origin/main\n', r'\1\n', content, flags=re.DOTALL)

with open('src/game.rs', 'w') as f:
    f.write(content)

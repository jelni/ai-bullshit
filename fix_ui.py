import re

with open('src/ui.rs', 'r') as f:
    content = f.read()

content = re.sub(r'(\s*"Speedrun Mode",\n)', r'\1        "Boss Rush Mode",\n', content)

with open('src/ui.rs', 'w') as f:
    f.write(content)

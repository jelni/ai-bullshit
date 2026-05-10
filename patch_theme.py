import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# Update Theme enum
content = re.sub(
    r'(Neon,\n\s*Ocean,\n)',
    r'\1    Cyberpunk,\n    Matrix,\n',
    content
)

# Update Theme::next
content = re.sub(
    r'(Self::Neon => Self::Ocean,\n\s*Self::Ocean => Self::Classic,\n)',
    r'Self::Neon => Self::Ocean,\n            Self::Ocean => Self::Cyberpunk,\n            Self::Cyberpunk => Self::Matrix,\n            Self::Matrix => Self::Classic,\n',
    content
)

# Update Theme::prev
content = re.sub(
    r'(Self::Classic => Self::Ocean,\n)',
    r'Self::Classic => Self::Matrix,\n',
    content
)

content = re.sub(
    r'(Self::Ocean => Self::Neon,\n)',
    r'Self::Ocean => Self::Neon,\n            Self::Cyberpunk => Self::Ocean,\n            Self::Matrix => Self::Cyberpunk,\n',
    content
)

with open('src/game.rs', 'w') as f:
    f.write(content)

with open('src/ui.rs', 'r') as f:
    ui_content = f.read()

ui_content = re.sub(
    r'(crate::game::Theme::Ocean => \(Color::DarkBlue, Color::Yellow, Color::Cyan, Color::White,\),\n)',
    r'\1        crate::game::Theme::Cyberpunk => (Color::Magenta, Color::Cyan, Color::Yellow, Color::Red,),\n        crate::game::Theme::Matrix => (Color::DarkGreen, Color::Green, Color::Green, Color::DarkGreen,),\n',
    ui_content
)

with open('src/ui.rs', 'w') as f:
    f.write(ui_content)

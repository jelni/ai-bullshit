with open('src/game/game_state.rs', 'r') as f:
    content = f.read()
if 'ArtifactShrine' not in content:
    content = content.replace('BattlePass,', 'BattlePass,\n    ArtifactShrine,')
with open('src/game/game_state.rs', 'w') as f:
    f.write(content)

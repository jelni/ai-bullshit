with open('src/main.rs', 'r') as f:
    content = f.read()

# 1. Add match arm in handle_key_event
content = content.replace(
    'GameState::BattlePass => handle_battle_pass_input(code, game),',
    'GameState::BattlePass => handle_battle_pass_input(code, game),\n        GameState::ArtifactShrine => handle_artifact_shrine_input(code, game),'
)

# 2. Add handle_artifact_shrine_input
new_func = """
fn handle_artifact_shrine_input(code: KeyCode, game: &mut Game) -> bool {
    use rand::Rng;
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.stats.coins >= 1000 {
                game.stats.coins -= 1000;
                let artifacts = [
                    crate::game::Artifact::CoinAmulet,
                    crate::game::Artifact::LifeChalice,
                    crate::game::Artifact::GhostCloak,
                    crate::game::Artifact::MagnetStone,
                    crate::game::Artifact::TimeCrystal,
                ];
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..artifacts.len());
                let artifact = artifacts[idx];

                if !game.stats.unlocked_artifacts.contains(&artifact) {
                    game.stats.unlocked_artifacts.push(artifact);
                } else {
                    game.stats.coins += 500;
                }
                game.save_stats();
                crate::game::beep();
            }
        },
        _ => {},
    }
    true
}
"""
content += new_func

# 3. Update handle_menu_input
content = content.replace(
    '54 => {\n                game.previous_state = Some(GameState::Menu);\n                game.state = GameState::ConfirmQuit;\n            },',
    '54 => {\n                game.state = GameState::ArtifactShrine;\n            },\n            55 => {\n                game.previous_state = Some(GameState::Menu);\n                game.state = GameState::ConfirmQuit;\n            },'
)

# Fix limits for menu selection
content = content.replace('game.menu_selection = 54;', 'game.menu_selection = 55;')
content = content.replace('if game.menu_selection < 54 {', 'if game.menu_selection < 55 {')

with open('src/main.rs', 'w') as f:
    f.write(content)

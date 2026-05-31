use snake_game::game::{Game, GameState, Theme, Difficulty};
use crossterm::event::KeyCode;

// Function from main.rs to test
fn handle_battle_pass_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 49; // 50 tiers
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 49 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let tier = (game.settings_selection + 1) as u32;
            let required_xp = tier * 1000;
            if game.stats.battle_pass_xp >= required_xp && !game.stats.claimed_battle_pass_tiers.contains(&tier) {
                // Claim reward
                game.stats.claimed_battle_pass_tiers.push(tier);

                // Determine reward based on tier
                if tier % 10 == 0 {
                    // Big reward (Skin or Theme)
                    if tier == 50 {
                        if !game.stats.unlocked_skins.contains(&'🚀') {
                            game.stats.unlocked_skins.push('🚀');
                        }
                    } else {
                        game.stats.coins += 5000;
                    }
                } else if tier % 5 == 0 {
                    game.stats.coins += 2000;
                } else {
                    game.stats.coins += 500;
                }
            }
        },
        _ => {},
    }
    true
}

#[test]
fn test_battle_pass_xp_gain() {
    let mut game = Game::new(20, 20, false, 'X', Theme::Classic, Difficulty::Normal);
    assert_eq!(game.stats.battle_pass_xp, 0);
    game.gain_xp(50);
    assert_eq!(game.stats.battle_pass_xp, 50);
    game.gain_xp(1000);
    assert_eq!(game.stats.battle_pass_xp, 1050);
}

#[test]
fn test_battle_pass_claim_reward() {
    let mut game = Game::new(20, 20, false, 'X', Theme::Classic, Difficulty::Normal);
    game.state = GameState::BattlePass;
    game.settings_selection = 0; // Tier 1

    // Not enough XP
    game.stats.battle_pass_xp = 500;
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert!(!game.stats.claimed_battle_pass_tiers.contains(&1));
    assert_eq!(game.stats.coins, 0); // No reward

    // Enough XP
    game.stats.battle_pass_xp = 1000;
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert!(game.stats.claimed_battle_pass_tiers.contains(&1));
    assert_eq!(game.stats.coins, 500); // 500 coins for non-multiple of 5

    // Trying to claim again
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert_eq!(game.stats.coins, 500); // Still 500

    // Tier 5 reward
    game.settings_selection = 4; // Tier 5
    game.stats.battle_pass_xp = 5000;
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert!(game.stats.claimed_battle_pass_tiers.contains(&5));
    assert_eq!(game.stats.coins, 2500); // 500 + 2000

    // Tier 10 reward
    game.settings_selection = 9; // Tier 10
    game.stats.battle_pass_xp = 10000;
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert!(game.stats.claimed_battle_pass_tiers.contains(&10));
    assert_eq!(game.stats.coins, 7500); // 2500 + 5000

    // Tier 50 reward
    game.settings_selection = 49; // Tier 50
    game.stats.battle_pass_xp = 50000;
    assert!(!game.stats.unlocked_skins.contains(&'🚀'));
    handle_battle_pass_input(KeyCode::Enter, &mut game);
    assert!(game.stats.claimed_battle_pass_tiers.contains(&50));
    assert!(game.stats.unlocked_skins.contains(&'🚀')); // Got skin

    // Scroll down
    game.settings_selection = 0;
    handle_battle_pass_input(KeyCode::Up, &mut game);
    assert_eq!(game.settings_selection, 49);

    // Scroll up
    handle_battle_pass_input(KeyCode::Down, &mut game);
    assert_eq!(game.settings_selection, 0);
}

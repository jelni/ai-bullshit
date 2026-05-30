#!/bin/bash
sed -i 's/        GameState::CompanionCamp => handle_companion_camp_input(code, game),/        GameState::CompanionCamp => handle_companion_camp_input(code, game),\n        GameState::ClassSelect => handle_class_select_input(code, game),/' src/main.rs

cat << 'INNER_EOF' >> src/main.rs

fn handle_class_select_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 4;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 4 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Warrior) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Warrior);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Warrior);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Warrior);
                }
            },
            1 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Mage) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Mage);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Mage);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Mage);
                }
            },
            2 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Rogue) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Rogue);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Rogue);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Rogue);
                }
            },
            3 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Paladin) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Paladin);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Paladin);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Paladin);
                }
            },
            4 => {
                game.stats.equipped_class = None;
            },
            _ => {},
        },
        _ => {},
    }
    game.save_high_scores();
    true
}
INNER_EOF

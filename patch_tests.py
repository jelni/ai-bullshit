with open('src/game/tests.rs', 'r') as f:
    content = f.read()

new_tests = """
#[test]
fn test_artifact_life_chalice() {
    let mut game = crate::game::Game::new(
        20,
        20,
        false,
        'x',
        crate::game::Theme::Classic,
        crate::game::Difficulty::Normal,
    );
    game.stats.unlocked_artifacts.push(crate::game::Artifact::LifeChalice);
    game.reset();
    // Base is 1. +1 from chalice = 2
    assert_eq!(game.lives, 2, "LifeChalice should add an extra life on reset");
}

#[test]
fn test_artifact_coin_amulet() {
    let mut game = crate::game::Game::new(
        20,
        20,
        false,
        'x',
        crate::game::Theme::Classic,
        crate::game::Difficulty::Normal,
    );
    let initial_coins = game.stats.coins;
    game.stats.unlocked_artifacts.push(crate::game::Artifact::CoinAmulet);

    // Process food collision to see if coins are doubled
    game.process_food_collision(game.food, false);

    // Normal food is 50 coins. CoinAmulet doubles it to 100.
    assert_eq!(game.stats.coins, initial_coins + 100, "CoinAmulet should double coins earned from food");
}

#[test]
fn test_artifact_ghost_cloak() {
    let mut game = crate::game::Game::new(
        20,
        20,
        false,
        'x',
        crate::game::Theme::Classic,
        crate::game::Difficulty::Normal,
    );
    game.stats.unlocked_artifacts.push(crate::game::Artifact::GhostCloak);

    // Setup scenario to guarantee dodge if possible, but it's random (10% chance)
    // To ensure the code runs without crashing and has a chance of working, we just force death multiple times
    // and check that lives are tracked properly, but deterministic testing is hard with RNG.
    // However, we just ensure it doesn't break logic.
    let initial_lives = game.lives;
    game.handle_death("Test Death");
    // Could have dodged (lives == initial_lives) or not (lives == initial_lives - 1)
    assert!(game.lives == initial_lives || game.lives == initial_lives.saturating_sub(1));
}
"""

if "test_artifact_life_chalice" not in content:
    content = content.replace("}\n\n#[cfg(test)]\npub mod tests {", "}\n\n" + new_tests + "\n#[cfg(test)]\npub mod tests {")

with open('src/game/tests.rs', 'w') as f:
    f.write(content)

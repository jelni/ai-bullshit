use snake_game::game::{CompanionType, Difficulty, Game, HeroClass, Theme};

#[test]
fn test_ranger_class_spawns_sniper() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);

    // Default companion is none (or whatever defaults are setup in stats, let's clear it just in case)
    game.stats.equipped_companion = None;

    // Equip Ranger class
    game.stats.equipped_class = Some(HeroClass::Ranger);

    // Reset game to trigger companion spawning logic
    game.reset();

    // Verify companion was spawned and is a Sniper
    assert!(game.companion.is_some(), "Companion should have spawned for Ranger class");
    let companion = game.companion.unwrap();
    assert_eq!(companion.kind, CompanionType::Sniper, "Companion should be a Sniper");
}

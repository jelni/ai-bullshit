I will implement a "Hero Class System" for the player snake. This expands the RPG aspect of the game.

1. Add `HeroClass` enum (`Warrior`, `Mage`, `Rogue`, `Paladin`) in `src/game/hero_class.rs`.
2. Expose it in `src/game/mod.rs`.
3. Add `pub unlocked_classes: Vec<HeroClass>` and `pub equipped_class: Option<HeroClass>` to `Statistics`.
4. Add `GameState::ClassSelect` to select classes.
5. In `Game::update()`, add class-specific effects:
    - **Warrior**: +1 Damage to bosses, passive life regen every 100 score.
    - **Mage**: Can freeze lasers, lasers home in on enemies.
    - **Rogue**: Faster movement, occasionally dodges boss lasers.
    - **Paladin**: Shields periodically spawn that absorb 1 hit.
6. Make a UI for `GameState::ClassSelect`.
7. Pre-commit hooks.

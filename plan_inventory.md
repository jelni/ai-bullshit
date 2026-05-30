# Plan: Implement Inventory & Crafting System

1. Add `Resource` types: `Wood`, `Iron`, `Gold`, `Diamond`.
2. Add `CraftableItem` types: `SpeedPotion`, `IronWall`, `GoldenApple`, `DiamondSword`.
3. Add `inventory: HashMap<Resource, u32>` and `crafted_items: HashMap<CraftableItem, u32>` to `Statistics` (so it persists).
4. Add `resources: HashMap<Point, Resource>` to `Game` struct for active resources on board.
5. In `Game::update()`, add `manage_resources()` to occasionally spawn resources.
6. When Snake head hits a resource, remove it and add to `stats.inventory`.
7. Add `GameState::Crafting` menu. Reachable from `Menu` or maybe mid-game via `C`.
8. In `GameState::Crafting`, display recipes:
   - Speed Potion: 3 Wood
   - Iron Wall: 3 Iron
   - Golden Apple: 5 Gold
   - Diamond Sword: 1 Diamond
9. Player can craft items.
10. In `Playing` state, add hotkeys `1`, `2`, `3`, `4` to consume crafted items.
    - `1`: Speed Potion -> Sets powerup `SpeedBoost`.
    - `2`: Iron Wall -> Spawns an obstacle behind snake.
    - `3`: Golden Apple -> Grants an Extra Life.
    - `4`: Diamond Sword -> Kills the nearest boss instantly.

Let's refine this. `manage_resources` would work just like `manage_bonus_food`.
We need to update `ui.rs` to draw the resources (maybe Wood: 'W', Iron: 'I', Gold: 'G', Diamond: 'D').
Wait, `G` is already used for Goblin? We can use 'T' for Wood, 'I' for Iron, 'A' for Gold (Au), 'D' for Diamond. Or use colored symbols:
Wood: 🪵 (or 'W' brown)
Iron: 🔗 (or 'I' gray)
Gold: 💰 (or 'G' yellow, Goblin uses 'G' though so maybe 'O')
Diamond: 💎 (or 'D' cyan)

This is a very cool feature! It fits the "massive multiplayer e-sports RPG" vision perfectly.

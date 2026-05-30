# Plan for Farmstead Mode
1. **Update `GameMode`**: Add `Farmstead` to `src/game/game_mode.rs`.
2. **Add `Crop` struct**: In `src/game/crop.rs` (or inside `mod.rs`), representing a planted seed with `position`, `growth_stage` (0=seed, 1=sprout, 2=mature), and `timer`.
3. **Update `Game` struct**:
   - Add `pub crops: Vec<Crop>` to `Game`.
   - Add `pub seed_inventory: u32` to `Statistics` (or just use coins). Let's use coins for simplicity, planting a seed costs 10 coins.
4. **Update `Game::update`**:
   - Manage crops: increment their timers. If a crop reaches mature stage, it acts like a bonus food! Wait, maybe it stays on the map until eaten.
   - Pests: occasionally spawn `Goblin` that paths towards the crops.
5. **Update Input Handling**:
   - In `Farmstead` mode, press `C` or `5` to plant a seed at the snake's tail or head? At the tail is better (like dropping a mine) or just spawn it on the spot.
6. **Update Drawing (`src/ui.rs`)**:
   - Draw crops based on their stage (e.g., `.` for seed, `v` for sprout, `T` or `¥` for mature).
7. **Menu Integration**: Add `Farmstead Mode` to `draw_menu` and `handle_menu_input`.

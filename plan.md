1. **Fix missing logic for `CenturyChallenge`, `MillenniumChallenge`, and `EonChallenge` in `src/main.rs`**
   - The arrow key handling inside `src/main.rs` contains an `if` condition to dispatch movement input. `CenturyChallenge`, `MillenniumChallenge`, and `EonChallenge` are missing and must be added to `KeyCode::Up`, `KeyCode::Down`, `KeyCode::Left`, and `KeyCode::Right`.
2. **Fix missing logic for all challenge modes (from `WeeklyChallenge` through `EonChallenge`) in `src/game/game_struct.rs`**
   - In `reset()`: Around line 1969, `WeeklyChallenge`, `MonthlyChallenge`, `YearlyChallenge`, `DecadeChallenge`, `CenturyChallenge`, `MillenniumChallenge`, `EonChallenge` are completely missing.
   - In `reset()`: Around line 2003, `WeeklyChallenge` through `EonChallenge` are missing from the `ref_snake` setup condition.
   - In `update()`: Around line 7447, `MonthlyChallenge`, `YearlyChallenge`, `DecadeChallenge`, `CenturyChallenge`, `MillenniumChallenge`, `EonChallenge` are missing from the game over check (`self.handle_death`).
   - In `add_obstacles_if_needed()`: Around line 8521, `MonthlyChallenge` through `EonChallenge` are missing.
3. **Verify the plan using `cargo test`, `cargo clippy`, and pre-commit instructions**

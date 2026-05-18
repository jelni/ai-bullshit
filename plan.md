1. **Understand the Goal**: Add poison food avoidance to the pathfinding logic. The test `test_bot_avoids_poison_food` ensures the bot doesn't eat poison food. In `src/game.rs`, inside `fn astar_search`, there is already entity avoidance code for bosses, lasers, mines, black holes, and meteors. We need to add a penalty for being close to poison food so the bot avoids it during pathfinding.
2. **Add Poison Food Avoidance to A* Heuristic**:
   - Open `src/game.rs`.
   - Locate `fn astar_search` and its inner `heuristic` closure.
   - Below the other entity avoidances (e.g., around line 4647), add a check for `self.poison_food`.
   - If `self.poison_food` is `Some((pf_p, _))`, calculate the distance `d` between `p` and `pf_p`.
   - If `d < 4`, add a penalty: `penalty = penalty.saturating_add((4 - d) * 10);`. This ensures the bot will prioritize paths that don't pass adjacent to poison food.
3. **Pre commit instructions**:
   - Run `cargo test` to ensure `test_bot_avoids_poison_food` and all other tests pass.
   - Run `./clippy.sh`
4. **Submit**:
   - Use `submit` to push the changes.

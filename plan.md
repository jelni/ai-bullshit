1. **Understand the Goal**: The issue mentions a new "Mines" feature was added recently (`mines: HashSet<Point>`). Currently, the pathfinding logic (`astar_search`) has penalties for `boss` and `lasers` but *not* for `mines`. I need to make the AI bot actively avoid `mines` by adding a similar penalty in the `heuristic` function of `astar_search`.
2. **Add Mine Avoidance to A* Heuristic**:
   - In `src/game.rs`, inside `fn astar_search`, locate the `heuristic` closure.
   - Add a check for being close to any mine in `self.mines`.
   - Calculate a penalty if the distance `d` is less than some threshold (e.g., 3 or 4) to discourage the bot from walking near mines. Note that direct collisions with mines are already prevented by `is_safe_final_p`, but adding a penalty makes the bot path *around* them gracefully. Wait, let's look at `is_safe_final_p` - it already returns `false` if `self.mines.contains(&final_p)`. Adding a penalty in `heuristic` for mines that are nearby will help it avoid getting boxed in. Let's add a penalty for distance `< 4`.
3. **Pre-commit Instructions**:
   - Run tests to verify the `test_bot_avoids_mines` or similar passes.
   - Run `./clippy.sh` and make sure it succeeds.
4. **Submit**:
   - Submit the changes with an appropriate message.

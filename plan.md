1. **Update A* Target Generation in `src/game.rs`**
   - The bot should explicitly target the goblin.
   - Update `calculate_autopilot_move` and `calculate_p2_autopilot_move` to push `goblin.position` to `targets`.

2. **Pre Commit Steps**
   - Ensure proper testing, verification, review, and reflection are done by calling the `pre_commit_instructions` tool and executing its steps.

3. **Submit Changes**
   - Commit and submit the code.

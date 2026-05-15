1. **Goal**: The next major development is an improvement to the foundational AI engine that powers the Bot Modes. Currently, the A* bot (used in `PlayerVsBot` and `BotVsBot` modes) does not predict the dynamic movement of the opponent snake. This causes "dumb" collisions where the bot drives directly into a space the opponent is moving into.
2. **Predictive Opponent Avoidance**:
    - Update `is_safe_final_p` in `src/game.rs` to dynamically predict the movement of `player2` (if checking for Player 1) and `snake` (if checking for Player 2).
    - If `checking_player` is 1, look at `self.player2`. If `player2` is heading in a `direction`, its head will be at `head + direction` in `steps=1`. We should assume the opponent's head could be anywhere in its immediate forward path or we should avoid its potential next positions.
    - Since we don't perfectly know where the opponent will turn, avoiding its straight path is a good start. Or better, avoid the cells adjacent to the opponent's head if `steps == 1` to be safe from "head-on" collisions.
3. **Changes**:
    - Pass `checking_player` context effectively into `is_safe_final_p`.
    - If `checking_player == 1`, and `steps == 1`, check if `final_p` is adjacent to `player2.head()`. If it is, and we could collide, avoid it. However, if it's the *only* way, maybe we don't avoid it. But let's just make it unsafe if it's the cell directly in front of `player2`.
    - Actually, `player2` has a `direction`. Its next head position is guaranteed to be one of the valid moves (straight, left turn, right turn). The most likely is straight.
    - Let's conservatively mark `p.head() + p.direction` as unsafe for `steps == 1`.
    - Do the same for `player 2` avoiding `player 1` when `checking_player == 2`.
4. **Testing**:
    - The new test `test_bot_predicts_p2_movement` should pass!

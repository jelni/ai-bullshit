1. **Goal**: Add predictive opponent avoidance to the A* bot algorithm. The bot currently collides with the opponent (`player2`) or `snake` in `BotVsBot` mode because it does not predict where the opponent will move next.
2. **Predictive Opponent Avoidance**:
    - Update `is_safe_final_p` in `src/game.rs` to dynamically predict the movement of `player2` (if checking for Player 1) and `snake` (if checking for Player 2).
    - If `checking_player` is 1, look at `self.player2`. If `player2` is heading in a `direction`, its head will be at `head + direction` in `steps=1` or `get_final_p(head + direction)`.
    - Also need to predict if the opponent will enter a portal.
3. **Changes**:
    - In `is_safe_final_p`, if `checking_player == 1`, get `player2`. If `player2` exists, calculate its next position `next_p2_head = player2.head() + player2.direction`. Apply `get_final_p(next_p2_head)`. If `final_p` equals this predicted head, and `steps == 1`, return `false` (it's unsafe).
    - Do the same for `player 2` avoiding `player 1` (`self.snake`) when `checking_player == 2`.
    - Wait, the opponent can change its direction if it's currently at an intersection or wants to turn. But assuming it goes straight or avoiding its *potential* next moves (straight, left, right) might be too restrictive. Let's just assume it goes straight for now, or maybe check all valid 3 moves and avoid them all if `steps == 1`?
    - If we check all 3 valid moves, the bot will be very cautious. Let's just avoid its current `direction` as a simple, effective heuristic. Or, even better, check the distance.
    - If `player2` has a head, and distance from `player2.head()` to `final_p` is 1 (i.e., it's a neighbor), AND it's a valid move for `player2` (not backwards), maybe avoid it. But the prompt specifically asks to pass `test_bot_predicts_p2_movement` where P2 goes Up.
    - Let's specifically avoid `get_final_p(opponent.head() + opponent.direction)` when `steps == 1`.
4. **Pre-commit**: Follow `AGENTS.md` and `pre_commit_instructions` to ensure quality.
5. **Submit**.

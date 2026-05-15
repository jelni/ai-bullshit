1. **Goal**: The AI agent collides with lasers and bosses because it doesn't correctly predict their dynamic movement. My initial fix solved the static vs dynamic problem but broke `test_calculate_autopilot_avoids_laser` because it didn't account for "crossing paths" with lasers.
2. **Refining Laser Prediction in `is_safe_final_p`**:
   - A laser starts at `l.position`. In `steps` ticks, it moves `steps * 2` times.
   - We need to check if the laser passes through `final_p` *during* the tick `steps`.
   - Also, if the laser passes through `final_p` at *any time* before or at `steps`, does it destroy itself? If it hits a wall, yes. But we can simplify: if the laser has passed through `final_p` in the past, maybe it's safe now?
   - Actually, if we are checking `final_p` at `steps`, the snake arrives at `final_p` at tick `steps`.
   - Where is the laser at tick `steps`? It occupies `pos_1` and `pos_2` (the two positions it moves to during that tick).
   - What if the snake and the laser cross paths? At tick `steps - 1`, the snake is at `prev_p` and moves to `final_p`. During tick `steps`, the laser moves from `laser_prev_p` to `pos_1` to `pos_2`. If the laser passes through `prev_p` or `final_p` during this transition, they collide.
   - Wait, `is_safe_final_p` doesn't know `prev_p`. But we know `steps`!
   - A simpler, more robust check: Just consider the laser's path for the next `steps` ticks. If `final_p` is *anywhere* on the laser's path from tick `0` up to tick `steps`, and the laser would be there at the same time or if it crosses paths...
   - Actually, a laser only exists for a short time before hitting a wall.
   - If we check if `final_p` is *on the laser's ray*, and the laser will reach `final_p` at time `t`, we can compare `t` with `steps`.
   - The laser moves 2 units per tick. It reaches a distance `d` from its origin in `tick = ceiling(d / 2)` ticks.
   - So if `final_p` is on the laser's ray at distance `d`:
     - The laser arrives at `final_p` at tick `ceil(d / 2)`.
     - The snake arrives at `final_p` at tick `steps`.
     - If `ceil(d / 2) == steps`, they collide directly at `final_p`!
     - What if they cross paths? The snake moves from `prev_p` (dist `d-1` or `d+1`) to `final_p`. The laser moves from `p1` to `p2`.
     - If `steps == ceil(d/2)`, they might cross. Actually, if `ceil(d/2) == steps` or `ceil(d/2) == steps + 1`, they might be adjacent and cross.
     - Just consider the whole ray: if `final_p` is on the ray at distance `d`:
       - If `d <= steps * 2 + 1` (the laser can reach it within `steps` ticks, plus 1 to account for passing through), then we should probably just avoid `final_p` entirely if it's anywhere near the laser's future path at that time.
       - Actually, a laser is so fast and dangerous, we can just say: if `final_p` is on the laser's path, and `d / 2 <= steps + 1`, it's unsafe. Or even simpler: if `final_p` is on the ray, and `steps <= d`, then the laser will hit it? No, if `steps > d/2`, the snake arrives *after* the laser has passed, which is safe.
       - Let's trace it:
         - Distance `d` from laser to `final_p`.
         - Time for laser to reach `final_p`: `t_l = ceil(d / 2.0)`.
         - Time for snake to reach `final_p`: `t_s = steps`.
         - If `t_l == t_s`, exact collision.
         - If `t_s == t_l` or `t_s == t_l + 1` or `t_s == t_l - 1` (to account for path crossing), it's unsafe.
         - Let's be very conservative but simple: if `final_p` is on the laser's ray, and the laser moves *towards* `final_p`, let `d` be the distance. If `steps * 2 >= d` and `steps * 2 <= d + 2`, they might cross.
   - What if we just simulate the laser and check if `final_p` is in the laser's current segment OR previous segment?
   - Let's refine the laser ray check:
     ```rust
     for l in &self.lasers {
         let dx = i32::from(final_p.x) - i32::from(l.position.x);
         let dy = i32::from(final_p.y) - i32::from(l.position.y);
         let on_ray = match l.direction {
             Direction::Up => dx == 0 && dy < 0,
             Direction::Down => dx == 0 && dy > 0,
             Direction::Left => dy == 0 && dx < 0,
             Direction::Right => dy == 0 && dx > 0,
         };
         if on_ray {
             let d = dx.abs().max(dy.abs()) as u16;
             // Laser reaches distance d at tick ceil(d/2.0)
             // Snake reaches final_p at tick `steps`
             // Also need to consider if laser passes through the snake.
             // If snake is at final_p at tick `steps`, and was at `prev_p` at `steps-1`.
             // It means snake occupies final_p at tick `steps`.
             // Laser occupies d at tick `(d+1)/2`.
             let laser_tick = (d + 1) / 2;
             if laser_tick == steps {
                 return false;
             }
             // What if they cross? Snake goes to d, laser goes from d+1 to d-1.
             // Snake is at d at tick `steps`. Laser is at d+1, d, d-1 at tick `steps`.
             // So if laser_tick == steps, it hits.
             // What if snake is at d+1 at tick steps? Laser is at d+1 at tick steps.
             // So we just check if distance d is reached by laser at tick `steps`.
             // The laser covers distances `steps*2 - 1` and `steps*2` at tick `steps`.
             // If `d == steps*2 - 1` or `d == steps*2`, they collide at `final_p`.
             if d == steps * 2 - 1 || d == steps * 2 {
                 return false;
             }
             // What about crossing paths?
             // Snake at `steps-1` was at distance `d±1`.
             // Laser at `steps-1` was at distance `steps*2 - 3` and `steps*2 - 2`.
             // Snake at `steps` is at distance `d`.
             // Laser at `steps` is at distance `steps*2 - 1` and `steps*2`.
             // If snake moves towards laser: snake goes from d+1 to d.
             // Laser goes from d-2 to d. (Cross!)
             // If `d == steps * 2 - 2` or `d == steps * 2 + 1`, they might cross.
             // Let's just say if `(steps*2).abs_diff(d) <= 2`, it's unsafe.
             if (steps * 2).abs_diff(d) <= 2 {
                 return false;
             }
             // Wait, what if the laser is currently right next to the snake? `d=1`. `steps=1`.
             // `steps*2 = 2`. `abs_diff(1) = 1 <= 2`. Unsafe. Correct.
         }
     }
     ```

3. **Check the Boss Collision Logic again:**
   - Previous logic:
     ```rust
     let move_threshold = if self.mode == GameMode::BossRush { ... } else { 2 } as u32;
     let moves = (u32::from(steps) + u32::from(boss.move_timer)) / move_threshold;
     let dist = u32::from(final_p.x.abs_diff(boss.position.x)) + u32::from(final_p.y.abs_diff(boss.position.y));
     if dist <= moves {
         return false;
     }
     ```
   - This seems robust and simple. Let's make sure it doesn't fail existing tests.

4. **Boss Laser Logic**:
   - Previous logic:
     ```rust
     let shoot_threshold = if self.mode == GameMode::BossRush { ... } else { 15 } as u32;
     let shoots = (u32::from(steps) + u32::from(boss.shoot_timer)) / shoot_threshold;
     if shoots > 0 {
         if final_p.x == boss.position.x || final_p.y == boss.position.y {
             return false;
         }
     }
     ```
   - Actually, if the boss shoots a laser, it travels at 2 units per tick. So it won't instantly hit the snake unless the snake is very close. But it's safer to just avoid the cross line of the boss if it's about to shoot.

5. **Write `plan.md`** to request review.

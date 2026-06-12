import re

with open("src/game/game_struct.rs", "r") as f:
    content = f.read()

# Let's write the exact block correctly.
search = """                    let check_laser_threat = |laser_pos: Point, dist_offset: u32| -> bool {
                        let dx = i32::from(final_p.x) - i32::from(laser_pos.x);
                        let dy = i32::from(final_p.y) - i32::from(laser_pos.y);
                        let on_ray = match l.direction {
                            Direction::Up => dx == 0 && dy <= 0,
                            Direction::Down => dx == 0 && dy >= 0,
                            Direction::Left => dy == 0 && dx <= 0,
                            Direction::Right => dy == 0 && dx >= 0,
                        };
                        if on_ray {
                            let d = u32::try_from(dx.abs().max(dy.abs())).unwrap_or(0) + dist_offset;
                            let step_dist = u32::from(steps) * 2;
                            if step_dist.abs_diff(d) <= 2 {
                                return true;
                            }
                        }
                        false
                    };"""

replace = """                    let check_laser_threat = |laser_pos: Point, dist_offset: u32| -> bool {
                        let dx = i32::from(final_p.x) - i32::from(laser_pos.x);
                        let dy = i32::from(final_p.y) - i32::from(laser_pos.y);
                        let on_ray = match l.direction {
                            Direction::Up => dx == 0 && dy <= 0,
                            Direction::Down => dx == 0 && dy >= 0,
                            Direction::Left => dy == 0 && dx <= 0,
                            Direction::Right => dy == 0 && dx >= 0,
                        };
                        if on_ray {
                            let d = u32::try_from(dx.abs().max(dy.abs())).unwrap_or(0) + dist_offset;
                            let step_dist = u32::from(steps) * 2;

                            // A laser travels 2 units per step.
                            // If `d` is smaller than `step_dist`, the laser has already passed this point
                            // or is passing it this step.
                            // However, we only care if the laser hits us AT `steps` or after (if it's a constant threat).
                            // A better approximation is: does the laser reach or pass `final_p` in `steps` ticks?
                            if step_dist.abs_diff(d) <= 2 || d <= step_dist + 1 {
                                return true;
                            }
                        }
                        false
                    };"""

content = content.replace(search, replace)

with open("src/game/game_struct.rs", "w") as f:
    f.write(content)

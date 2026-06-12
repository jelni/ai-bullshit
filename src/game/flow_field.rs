use std::collections::{HashMap, VecDeque};
use crate::game::Game;
use crate::snake::{Direction, Point};

#[must_use]
pub fn generate_flow_field(game: &Game, targets: &[Point]) -> HashMap<Point, Direction> {
    let mut flow_field = HashMap::new();
    let mut queue = VecDeque::new();

    for &t in targets {
        queue.push_back((t, 0_u16));
    }

    let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

    while let Some((curr, dist)) = queue.pop_front() {
        for &d in &dirs {
            let opposite = match d {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };

            let mut candidates = vec![Game::calculate_next_head_dir(curr, opposite)];
            if let Some((p1, p2)) = game.portals {
                if curr == p1 { candidates.push(Game::calculate_next_head_dir(p2, opposite)); }
                if curr == p2 { candidates.push(Game::calculate_next_head_dir(p1, opposite)); }
            }

            if game.wrap_mode || game.mode == crate::game::GameMode::Zen {
                if curr.x == 1 && opposite == Direction::Left { candidates.push(Point { x: game.width - 2, y: curr.y }); }
                if curr.x == game.width - 2 && opposite == Direction::Right { candidates.push(Point { x: 1, y: curr.y }); }
                if curr.y == 1 && opposite == Direction::Up { candidates.push(Point { x: curr.x, y: game.height - 2 }); }
                if curr.y == game.height - 2 && opposite == Direction::Down { candidates.push(Point { x: curr.x, y: 1 }); }
            }

            for final_prev in candidates {
                // Check if final_prev is actually a valid point on the board
                let margin = if game.mode == crate::game::GameMode::BattleRoyale { game.safe_zone_margin } else { 0 };
                if final_prev.x <= margin || final_prev.x >= game.width - 1 - margin || final_prev.y <= margin || final_prev.y >= game.height - 1 - margin {
                    continue;
                }

                if !flow_field.contains_key(&final_prev) && !targets.contains(&final_prev) {
                    let next_from_prev = Game::calculate_next_head_dir(final_prev, d);
                    if let Some(final_curr_test) = game.get_final_p(next_from_prev) {
                        if final_curr_test == curr && !game.obstacles.contains(&final_prev) {
                            flow_field.insert(final_prev, d);
                            queue.push_back((final_prev, dist + 1));
                        }
                    }
                }
            }
        }
    }

    flow_field
}

import re
import sys

def main():
    with open('src/game/game_struct.rs', 'r') as f:
        content = f.read()

    # Step 1: Inject get_boss_path

    boss_path_code = """
    pub fn get_boss_path(&self, start: Point, target: Point, boss_kind: BossType) -> Option<Point> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut came_from = std::collections::HashMap::new();

        g_score.insert(start, 0u16);

        let heuristic = |p: Point| -> u16 {
            let dx = p.x.abs_diff(target.x);
            let dy = p.y.abs_diff(target.y);
            dx + dy
        };

        open_set.push(AStarState {
            f_score: heuristic(start),
            position: start,
        });

        while let Some(AStarState { position: current, .. }) = open_set.pop() {
            if current == target {
                let mut path = Vec::new();
                let mut curr = current;
                while let Some(&prev) = came_from.get(&curr) {
                    path.push(curr);
                    if prev == start {
                        break;
                    }
                    curr = prev;
                }
                return path.last().copied();
            }

            let current_g = *g_score.get(&current).unwrap_or(&u16::MAX);

            let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
            for (dx, dy) in dirs {
                let nx = i32::from(current.x) + dx;
                let ny = i32::from(current.y) + dy;

                let margin = if self.mode == GameMode::BattleRoyale { self.safe_zone_margin } else { 0 } + 1;

                if nx > i32::from(margin) && nx < i32::from(self.width) - 1 - i32::from(margin) &&
                   ny > i32::from(margin) && ny < i32::from(self.height) - 1 - i32::from(margin) {

                    let next_p = Point { x: nx as u16, y: ny as u16 };

                    let mut can_move = true;
                    if self.snake.body_map.contains_key(&next_p) {
                        can_move = false;
                    } else if self.obstacles.contains(&next_p) {
                        if boss_kind != BossType::Charger && boss_kind != BossType::Juggernaut {
                            can_move = false;
                        }
                    }

                    if can_move {
                        let tentative_g = current_g.saturating_add(1);
                        if tentative_g < *g_score.get(&next_p).unwrap_or(&u16::MAX) {
                            came_from.insert(next_p, current);
                            g_score.insert(next_p, tentative_g);
                            open_set.push(AStarState {
                                f_score: tentative_g.saturating_add(heuristic(next_p)),
                                position: next_p,
                            });
                        }
                    }
                }
            }
        }
        None
    }
"""
    if "pub fn get_boss_path" not in content:
        # inject before update_tick
        content = content.replace("fn update_tick(&mut self) {", boss_path_code + "\n    fn update_tick(&mut self) {")

    with open('src/game/game_struct.rs', 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()

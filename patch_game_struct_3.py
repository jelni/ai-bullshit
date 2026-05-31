import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# I am going to replace the generic `bfs_pathfind` and `Juggernaut` greedy code in the `update_tick` loop
# with my new `get_boss_path` implementation.

search_str = """                        let dir_opt = if boss.kind == BossType::Juggernaut {
                            let dx = i32::from(target_pos.x) - i32::from(boss.position.x);
                            let dy = i32::from(target_pos.y) - i32::from(boss.position.y);
                            if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    Some(Direction::Right)
                                } else {
                                    Some(Direction::Left)
                                }
                            } else if dy != 0 {
                                if dy > 0 {
                                    Some(Direction::Down)
                                } else {
                                    Some(Direction::Up)
                                }
                            } else {
                                None
                            }
                        } else if boss.kind == BossType::Charger {
                            let targets = vec![target_pos];
                            let dx = i32::from(target_pos.x) - i32::from(boss.position.x);
                            let dy = i32::from(target_pos.y) - i32::from(boss.position.y);
                            let fake_dir = if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    Direction::Right
                                } else {
                                    Direction::Left
                                }
                            } else {
                                if dy > 0 {
                                    Direction::Down
                                } else {
                                    Direction::Up
                                }
                            };
                            self.astar_search(boss.position, fake_dir, &targets, 3)
                                .map(|(d, _)| d)
                                .or_else(|| self.bfs_pathfind(boss.position, target_pos))
                        } else {
                            self.bfs_pathfind(boss.position, target_pos)
                        };"""

replace_str = """                        let dir_opt = if let Some(next_p) = self.get_boss_path(boss.position, target_pos, boss.kind) {
                            let dx = i32::from(next_p.x) - i32::from(boss.position.x);
                            let dy = i32::from(next_p.y) - i32::from(boss.position.y);
                            if dx > 0 {
                                Some(Direction::Right)
                            } else if dx < 0 {
                                Some(Direction::Left)
                            } else if dy > 0 {
                                Some(Direction::Down)
                            } else if dy < 0 {
                                Some(Direction::Up)
                            } else {
                                None
                            }
                        } else {
                            None
                        };"""

if search_str in content:
    content = content.replace(search_str, replace_str)
    print("Replaced!")
else:
    print("Not replaced.")

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

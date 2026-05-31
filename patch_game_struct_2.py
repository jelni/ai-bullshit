import re
import sys

def main():
    with open('src/game/game_struct.rs', 'r') as f:
        content = f.read()

    search_str = """                        let mut target_pos = snake_head;
                        if let Some(p2) = &self.player2 {
                            if boss.position.x.abs_diff(p2.head().x)
                                + boss.position.y.abs_diff(p2.head().y)
                                < boss.position.x.abs_diff(snake_head.x)
                                    + boss.position.y.abs_diff(snake_head.y)
                            {
                                target_pos = p2.head();
                            }
                        }

                        let dx = i32::from(target_pos.x) - i32::from(boss.position.x);
                        let dy = i32::from(target_pos.y) - i32::from(boss.position.y);

                        let mut next_pos = boss.position;
                        if dx.abs() > dy.abs() {
                            if dx > 0 {
                                next_pos.x += 1;
                            } else {
                                next_pos.x -= 1;
                            }
                        } else if dy > 0 {
                            next_pos.y += 1;
                        } else {
                            next_pos.y -= 1;
                        }"""

    replace_str = """                        let mut target_pos = snake_head;
                        if let Some(p2) = &self.player2 {
                            if boss.position.x.abs_diff(p2.head().x)
                                + boss.position.y.abs_diff(p2.head().y)
                                < boss.position.x.abs_diff(snake_head.x)
                                    + boss.position.y.abs_diff(snake_head.y)
                            {
                                target_pos = p2.head();
                            }
                        }

                        let mut next_pos = boss.position;
                        if let Some(np) = self.get_boss_path(boss.position, target_pos, boss.kind) {
                            next_pos = np;
                        } else {
                            let dx = i32::from(target_pos.x) - i32::from(boss.position.x);
                            let dy = i32::from(target_pos.y) - i32::from(boss.position.y);
                            if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    next_pos.x += 1;
                                } else {
                                    next_pos.x -= 1;
                                }
                            } else if dy > 0 {
                                next_pos.y += 1;
                            } else {
                                next_pos.y -= 1;
                            }
                        }"""

    if search_str in content:
        content = content.replace(search_str, replace_str)
        print("Successfully replaced boss movement block.")
    else:
        print("Could not find boss movement block. Trying regex...")
        match = re.search(r"let mut target_pos = snake_head;.*?next_pos\.y -= 1;\n\s+\}", content, re.DOTALL)
        if match:
            print("Found block with regex.")
            content = content[:match.start()] + replace_str + content[match.end():]
        else:
            print("Regex also failed.")

    with open('src/game/game_struct.rs', 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()

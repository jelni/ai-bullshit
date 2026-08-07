import re

def apply_diff(filepath, search, replace):
    with open(filepath, 'r') as file:
        content = file.read()
    if search not in content:
        print(f"Error: Could not find search string in {filepath}")
        return False
    content = content.replace(search, replace, 1)
    with open(filepath, 'w') as file:
        file.write(content)
    return True

search = """            for boss in &self.bosses {
                if targets.contains(&boss.position) {
                    continue;
                }
                let mut d = calc_dist(p, boss.position);
                if let Some((p1, p2)) = self.portals {
                    let d_via_p1 = calc_dist(p, p1)
                        .saturating_add(calc_dist(p2, boss.position))
                        .saturating_add(1);
                    let d_via_p2 = calc_dist(p, p2)
                        .saturating_add(calc_dist(p1, boss.position))
                        .saturating_add(1);
                    d = std::cmp::min(d, std::cmp::min(d_via_p1, d_via_p2));
                }
                if d < 6 {
                    penalty = penalty.saturating_add((6 - d) * 40); // increased penalty for boss avoidance
                }
            }"""

replace = """            for boss in &self.bosses {
                if targets.contains(&boss.position) {
                    continue;
                }
                let mut d = calc_dist(p, boss.position);
                if let Some((p1, p2)) = self.portals {
                    let d_via_p1 = calc_dist(p, p1)
                        .saturating_add(calc_dist(p2, boss.position))
                        .saturating_add(1);
                    let d_via_p2 = calc_dist(p, p2)
                        .saturating_add(calc_dist(p1, boss.position))
                        .saturating_add(1);
                    d = std::cmp::min(d, std::cmp::min(d_via_p1, d_via_p2));
                }
                if d < 10 { // massive entity avoidance range
                    penalty = penalty.saturating_add((10 - d) * 100); // massive penalty for boss avoidance
                }
            }"""

apply_diff("src/game/game_struct.rs", search, replace)

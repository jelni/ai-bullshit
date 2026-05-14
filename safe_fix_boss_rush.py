import re

with open('src/game.rs', 'r') as f:
    content = f.read()

# 1. Modify Game::reset() to clear obstacles and set campaign_level = 1 for BossRush
content = content.replace(
"""        if self.mode == GameMode::CustomLevel {
            self.obstacles.clear();
            self.editor_cursor = Some(Point {
                x: start_x,
                y: start_y,
            });
        } else if self.mode == GameMode::Campaign {""",
"""        if self.mode == GameMode::CustomLevel {
            self.obstacles.clear();
            self.editor_cursor = Some(Point {
                x: start_x,
                y: start_y,
            });
        } else if self.mode == GameMode::BossRush {
            self.campaign_level = 1;
            self.obstacles.clear();
        } else if self.mode == GameMode::Campaign {"""
)

# 2. Modify tick() logic to spawn boss in BossRush mode if none exists
boss_logic = """        if self.boss.is_none() && self.mode == GameMode::SinglePlayer && self.rng.gen_bool(0.005) {
            let margin = self.safe_zone_margin + 5;"""
boss_logic_new = """        if self.boss.is_none() && self.mode == GameMode::BossRush {
            let margin = self.safe_zone_margin + 5;
            let avoid =
                |p: &Point| self.obstacles.contains(p) || self.snake.body_map.contains_key(p);
            if let Some(pos) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                margin,
            ) {
                let health = 1 + self.campaign_level * 2;
                self.boss = Some(Boss {
                    position: pos,
                    health,
                    max_health: health,
                    move_timer: 0,
                    shoot_timer: 0,
                });
                self.campaign_level += 1;
                self.chat_log.push_back((
                    web_time::Instant::now(),
                    "BOSS INCOMING!".to_string(),
                    Color::Red,
                ));
            }
        } else if self.boss.is_none() && self.mode == GameMode::SinglePlayer && self.rng.gen_bool(0.005) {
            let margin = self.safe_zone_margin + 5;"""

content = content.replace(boss_logic, boss_logic_new)

# 3. Add test case
test_case_regex = r'(\s*)#\[test\]\n\s*fn test_calculate_autopilot_uses_portals\(\) \{'
test_case_new = r'''\1#[test]
    fn test_boss_rush_mode() {
        let mut game = Game::new(20, 20, false, 'x', crate::game::Theme::Dark, crate::game::Difficulty::Normal);
        game.mode = GameMode::BossRush;
        game.reset();
        assert_eq!(game.campaign_level, 1);

        game.tick();
        assert!(game.boss.is_some());
        assert_eq!(game.boss.as_ref().unwrap().health, 3);
        assert_eq!(game.campaign_level, 2);
    }

\1#[test]
    fn test_calculate_autopilot_uses_portals() {'''

# Find the end of the file/module to safely append test instead of weird regex
content = content.replace("""    #[test]
    fn test_calculate_autopilot_uses_portals() {""", """    #[test]
    fn test_boss_rush_mode() {
        let mut game = Game::new(20, 20, false, 'x', crate::game::Theme::Dark, crate::game::Difficulty::Normal);
        game.mode = GameMode::BossRush;
        game.reset();
        assert_eq!(game.campaign_level, 1);

        game.tick();
        assert!(game.boss.is_some());
        assert_eq!(game.boss.as_ref().unwrap().health, 3);
        assert_eq!(game.campaign_level, 2);
    }

    #[test]
    fn test_calculate_autopilot_uses_portals() {""")


with open('src/game.rs', 'w') as f:
    f.write(content)

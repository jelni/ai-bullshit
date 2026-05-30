#!/bin/bash

# We want to insert effects:
# Warrior: +2 lives in reset()
# Mage: Start with TimeFreeze powerup in reset()
# Rogue: 20% dodge chance when losing life.
# Paladin: Regens a life every 1000 steps.

# Let's see where to inject Warrior lives:
sed -i 's/        self.lives = if self.skin ==/        self.lives = if self.stats.equipped_class == Some(crate::game::HeroClass::Warrior) { 3 + u32::from(self.stats.upgrade_extra_lives) } else if self.skin ==/' src/game/game_struct.rs

# Mage starting with TimeFreeze:
sed -i '/self.power_up = None;/a\        if self.stats.equipped_class == Some(crate::game::HeroClass::Mage) {\n            self.power_up = Some(PowerUp {\n                p_type: PowerUpType::TimeFreeze,\n                location: Point { x: 0, y: 0 },\n                activation_time: Some(web_time::SystemTime::now().duration_since(web_time::SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()),\n            });\n        }' src/game/game_struct.rs

# Rogue dodge:
# Look for self.lives = self.lives.saturating_sub(1);
sed -i 's/self.lives = self.lives.saturating_sub(1);/if self.stats.equipped_class == Some(crate::game::HeroClass::Rogue) \&\& self.rng.gen_bool(0.2) { crate::game::beep(); } else { self.lives = self.lives.saturating_sub(1); }/' src/game/game_struct.rs

# Paladin regen:
# In update():
sed -i '/self.manage_weather();/a\        if self.stats.equipped_class == Some(crate::game::HeroClass::Paladin) \&\& self.score > 0 \&\& self.score % 100 == 0 \&\& self.snake.body.len() % 2 == 0 { self.lives = self.lives.saturating_add(1); }' src/game/game_struct.rs

import re

with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Fix the borrow checker error in AOE logic
# We need to collect the spawn text requests and apply them after the loop
old_code = """                for &b_idx in &boss_hits {
                    if let Some(b) = self.bosses.get_mut(b_idx) {
                        b.health = b.health.saturating_sub(2); // AOE deals extra damage
                        self.spawn_floating_text(
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            "-2".to_string(),
                            crate::color::Color::Red,
                        );
                    }
                }"""

new_code = """                let mut spawn_texts = Vec::new();
                for &b_idx in &boss_hits {
                    if let Some(b) = self.bosses.get_mut(b_idx) {
                        b.health = b.health.saturating_sub(2); // AOE deals extra damage
                        spawn_texts.push((
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            "-2".to_string(),
                            crate::color::Color::Red,
                        ));
                    }
                }
                for (x, y, text, color) in spawn_texts {
                    self.spawn_floating_text(x, y, text, color);
                }"""

content = content.replace(old_code, new_code)

old_code_2 = """                for (i, boss) in self.bosses.iter_mut().enumerate() {
                    if boss.position == laser.position {
                        let mut damage = 1;
                        if laser.player == 1
                            && self.stats.faction == Some(crate::game::Faction::CrimsonVipers)
                        {
                            damage += 1 + (self.stats.faction_rep / 5000);
                        }
                        boss.health = boss.health.saturating_sub(damage);
                        self.spawn_floating_text(
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            format!("-{}", damage),
                            crate::color::Color::Red,
                        );
                        if !is_piercing {
                            destroyed = true;
                        }
                        hit_boss_idx = Some(i);
                        break;
                    }
                }"""

new_code_2 = """                let mut damage_text = None;
                for (i, boss) in self.bosses.iter_mut().enumerate() {
                    if boss.position == laser.position {
                        let mut damage = 1;
                        if laser.player == 1
                            && self.stats.faction == Some(crate::game::Faction::CrimsonVipers)
                        {
                            damage += 1 + (self.stats.faction_rep / 5000);
                        }
                        boss.health = boss.health.saturating_sub(damage);
                        damage_text = Some((
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            format!("-{}", damage),
                            crate::color::Color::Red,
                        ));
                        if !is_piercing {
                            destroyed = true;
                        }
                        hit_boss_idx = Some(i);
                        break;
                    }
                }
                if let Some((x, y, text, color)) = damage_text {
                    self.spawn_floating_text(x, y, text, color);
                }"""

content = content.replace(old_code_2, new_code_2)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

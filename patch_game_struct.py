with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# 1. LifeChalice: In reset(), after self.lives = ...
search_lives = "        self.lives = if self.stats.equipped_class == Some(crate::game::HeroClass::Warrior) { 3 + u32::from(self.stats.upgrade_extra_lives) } else if self.skin == '💖' {"
replace_lives = "        self.lives = if self.stats.equipped_class == Some(crate::game::HeroClass::Warrior) { 3 + u32::from(self.stats.upgrade_extra_lives) } else if self.skin == '💖' {\n            3 + u32::from(self.stats.upgrade_extra_lives)\n        } else {\n            1 + u32::from(self.stats.upgrade_extra_lives)\n        };\n        if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::LifeChalice) {\n            self.lives += 1;\n        }"

if search_lives in content:
    content = content.split(search_lives)[0] + replace_lives + content.split(search_lives)[1].split("        };", 1)[1]

# 2. CoinAmulet: Double coins_earned
search_coins = "            if self.skin == '₿' {\n                coins_earned *= 2;\n            }"
replace_coins = "            if self.skin == '₿' {\n                coins_earned *= 2;\n            }\n            if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::CoinAmulet) {\n                coins_earned *= 2;\n            }"
content = content.replace(search_coins, replace_coins)

search_coins_2 = "        if self.skin == '₿' {\n            coins_earned *= 2;\n        }"
replace_coins_2 = "        if self.skin == '₿' {\n            coins_earned *= 2;\n        }\n        if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::CoinAmulet) {\n            coins_earned *= 2;\n        }"
content = content.replace(search_coins_2, replace_coins_2)

# 3. GhostCloak: Dodge death
search_dodge = """        if self.stats.equipped_class == Some(crate::game::HeroClass::Rogue)
            && self.rng.gen_bool(0.2)
        {
            crate::game::beep();
        } else {
            self.lives = self.lives.saturating_sub(1);
        }"""
replace_dodge = """        if self.stats.equipped_class == Some(crate::game::HeroClass::Rogue)
            && self.rng.gen_bool(0.2)
        {
            crate::game::beep();
        } else if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::GhostCloak)
            && self.rng.gen_bool(0.10)
        {
            crate::game::beep();
        } else {
            self.lives = self.lives.saturating_sub(1);
        }"""
content = content.replace(search_dodge, replace_dodge)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

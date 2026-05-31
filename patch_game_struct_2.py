with open('src/game/game_struct.rs', 'r') as f:
    content = f.read()

# Wait, `replace_lives` didn't execute because the search string wasn't exactly matching.
search_lives = "        self.lives = if self.stats.equipped_class == Some(crate::game::HeroClass::Warrior) {\n            3 + u32::from(self.stats.upgrade_extra_lives)\n        } else if self.skin == '💎' {\n            3 + u32::from(self.stats.upgrade_extra_lives) + 1\n        } else {\n            3 + u32::from(self.stats.upgrade_extra_lives)\n        };"

replace_lives = search_lives + "\n        if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::LifeChalice) {\n            self.lives += 1;\n        }"

content = content.replace(search_lives, replace_lives)

with open('src/game/game_struct.rs', 'w') as f:
    f.write(content)

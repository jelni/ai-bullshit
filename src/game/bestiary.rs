use crate::game::BossType;

#[must_use]
pub fn get_boss_lore(boss_type: &BossType, kills: u32) -> &'static str {
    if kills == 0 {
        return "??? (Defeat this boss to unlock lore)";
    }
    match boss_type {
        BossType::Shooter => if kills >= 5 { "A ruthless sniper. Rumored to have perfect aim." } else { "Fires deadly lasers from afar." },
        BossType::Charger => if kills >= 5 { "Moves incredibly fast. Cannot change direction quickly." } else { "Charges at high speed." },
        BossType::Spawner => if kills >= 5 { "Lays explosive mines. Always keeps its distance." } else { "Spawns mines." },
        BossType::Teleporter => if kills >= 5 { "Can blink across the arena instantly." } else { "Teleports frequently." },
        BossType::Splitter => if kills >= 5 { "Splits into smaller parts upon death." } else { "Splits into pieces." },
        BossType::Trapper => if kills >= 5 { "Leaves deadly obstacles in its wake to trap prey." } else { "Leaves traps behind." },
        BossType::Necromancer => if kills >= 5 { "Summons minions to do its bidding." } else { "Summons the dead." },
        BossType::ShadowClone => if kills >= 5 { "Mirrors the player's movement exactly." } else { "Copies your every move." },
        BossType::Mimic => if kills >= 5 { "Disguises itself as food. Don't be fooled!" } else { "Looks like a tasty snack." },
        BossType::Puffer => if kills >= 5 { "Expands in size, creating a massive impassable area." } else { "Inflates to block paths." },
        BossType::Juggernaut => if kills >= 5 { "Heavily armored. Can destroy obstacles effortlessly." } else { "A massive, unstoppable force." },
    }
}

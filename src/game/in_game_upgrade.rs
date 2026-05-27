use super::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InGameUpgrade {
    Multishot,
    Piercing,
    ExplosiveLasers,
    LaserSpeed,
    HomingLasers,
    DoubleCoins,
    Magnet,
    Turret,
}

impl InGameUpgrade {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Multishot => "Multishot",
            Self::Piercing => "Piercing",
            Self::ExplosiveLasers => "Explosive Lasers",
            Self::LaserSpeed => "Laser Speed",
            Self::HomingLasers => "Homing Lasers",
            Self::DoubleCoins => "Double Coins",
            Self::Magnet => "Magnet",
            Self::Turret => "Deploy Turret",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Multishot => "Shoot extra parallel lasers",
            Self::Piercing => "Lasers pass through obstacles and enemies",
            Self::ExplosiveLasers => "Lasers explode on impact",
            Self::LaserSpeed => "Increase laser travel speed",
            Self::HomingLasers => "Lasers steer towards bosses",
            Self::DoubleCoins => "Increase coin drops from food",
            Self::Magnet => "Attract food and powerups from afar",
            Self::Turret => "Automatically deploys a turret that shoots enemies",
        }
    }
}

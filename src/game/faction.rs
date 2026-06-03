use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Hash)]
pub enum Faction {
    CrimsonVipers,
    AzureCobras,
    EmeraldPythons,
}

impl Faction {
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::CrimsonVipers => "+1 Laser Damage (+1 per 5000 Rep)",
            Self::AzureCobras => "-10ms Base Tick Rate (-1ms per 1000 Rep)",
            Self::EmeraldPythons => "+10% Coins (+1% per 1000 Rep)",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CrimsonVipers => "Crimson Vipers",
            Self::AzureCobras => "Azure Cobras",
            Self::EmeraldPythons => "Emerald Pythons",
        }
    }
}

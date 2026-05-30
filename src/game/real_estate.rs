use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Property {
    Shack,
    Apartment,
    Mansion,
    Skyscraper,
}

impl Property {
    #[must_use]
    pub const fn cost(self) -> u32 {
        match self {
            Self::Shack => 100,
            Self::Apartment => 500,
            Self::Mansion => 2000,
            Self::Skyscraper => 10000,
        }
    }

    #[must_use]
    pub const fn income_per_second(self) -> u32 {
        match self {
            Self::Shack => 1,
            Self::Apartment => 6,
            Self::Mansion => 28,
            Self::Skyscraper => 160,
        }
    }
}

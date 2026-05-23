#[derive(Clone, Copy, Debug, serde :: Serialize, serde :: Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Insane,
    GodMode,
}
impl Difficulty {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Hard,
            Self::Hard => Self::Insane,
            Self::Insane => Self::GodMode,
            Self::GodMode => Self::Easy,
        }
    }
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::Easy => Self::GodMode,
            Self::Normal => Self::Easy,
            Self::Hard => Self::Normal,
            Self::Insane => Self::Hard,
            Self::GodMode => Self::Insane,
        }
    }
}

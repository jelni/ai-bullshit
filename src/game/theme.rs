#[derive(Clone, Debug, serde :: Serialize, serde :: Deserialize, PartialEq, Eq, Default, Copy)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Theme {
    #[default]
    Classic,
    Dark,
    Retro,
    Neon,
    Ocean,
    Matrix,
    Galactic,
    Premium,
    Cyberpunk,
    Rainbow,
    Hacker,
    Blockchain,
    Esports,
    Solar,
    Metaverse,
}

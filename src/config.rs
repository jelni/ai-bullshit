use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    width: Option<u16>,

    #[arg(long)]
    height: Option<u16>,

    #[arg(long)]
    wrap: Option<bool>,

    #[arg(long)]
    skin: Option<char>,

    #[arg(long)]
    theme: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub skin: char,
    pub theme: String,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: 20,
            wrap_mode: false,
            skin: '█',
            theme: String::from("classic"),
        }
    }
}

impl GameConfig {
    pub fn from_args(args: Args) -> Self {
        let default = Self::default();
        Self {
            width: args.width.unwrap_or(default.width),
            height: args.height.unwrap_or(default.height),
            wrap_mode: args.wrap.unwrap_or(default.wrap_mode),
            skin: args.skin.unwrap_or(default.skin),
            theme: args.theme.unwrap_or(default.theme),
        }
    }
}

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 40)]
    pub width: u16,

    #[arg(long, default_value_t = 20)]
    pub height: u16,

    #[arg(long, default_value_t = false)]
    pub wrap: bool,

    #[arg(long, default_value_t = '█')]
    pub skin: char,

    #[arg(long, default_value_t = String::from("classic"))]
    pub theme: String,
}

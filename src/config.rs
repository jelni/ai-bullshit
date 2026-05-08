#[derive(Debug, Clone)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub skin: char,
    pub theme: String,
}

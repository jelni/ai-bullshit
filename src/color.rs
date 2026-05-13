#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
}

#[cfg(feature = "cli")]
impl From<Color> for crossterm::style::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::Reset => crossterm::style::Color::Reset,
            Color::Black => crossterm::style::Color::Black,
            Color::DarkGrey => crossterm::style::Color::DarkGrey,
            Color::Red => crossterm::style::Color::Red,
            Color::DarkRed => crossterm::style::Color::DarkRed,
            Color::Green => crossterm::style::Color::Green,
            Color::DarkGreen => crossterm::style::Color::DarkGreen,
            Color::Yellow => crossterm::style::Color::Yellow,
            Color::DarkYellow => crossterm::style::Color::DarkYellow,
            Color::Blue => crossterm::style::Color::Blue,
            Color::DarkBlue => crossterm::style::Color::DarkBlue,
            Color::Magenta => crossterm::style::Color::Magenta,
            Color::DarkMagenta => crossterm::style::Color::DarkMagenta,
            Color::Cyan => crossterm::style::Color::Cyan,
            Color::DarkCyan => crossterm::style::Color::DarkCyan,
            Color::White => crossterm::style::Color::White,
            Color::Grey => crossterm::style::Color::Grey,
        }
    }
}

#[cfg(feature = "cli")]
impl From<crossterm::style::Color> for Color {
    fn from(c: crossterm::style::Color) -> Self {
        match c {
            crossterm::style::Color::Reset => Color::Reset,
            crossterm::style::Color::Black => Color::Black,
            crossterm::style::Color::DarkGrey => Color::DarkGrey,
            crossterm::style::Color::Red => Color::Red,
            crossterm::style::Color::DarkRed => Color::DarkRed,
            crossterm::style::Color::Green => Color::Green,
            crossterm::style::Color::DarkGreen => Color::DarkGreen,
            crossterm::style::Color::Yellow => Color::Yellow,
            crossterm::style::Color::DarkYellow => Color::DarkYellow,
            crossterm::style::Color::Blue => Color::Blue,
            crossterm::style::Color::DarkBlue => Color::DarkBlue,
            crossterm::style::Color::Magenta => Color::Magenta,
            crossterm::style::Color::DarkMagenta => Color::DarkMagenta,
            crossterm::style::Color::Cyan => Color::Cyan,
            crossterm::style::Color::DarkCyan => Color::DarkCyan,
            crossterm::style::Color::White => Color::White,
            crossterm::style::Color::Grey => Color::Grey,
            _ => Color::White,
        }
    }
}

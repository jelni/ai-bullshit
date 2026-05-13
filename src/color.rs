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
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::DarkGrey => Self::DarkGrey,
            Color::Red => Self::Red,
            Color::DarkRed => Self::DarkRed,
            Color::Green => Self::Green,
            Color::DarkGreen => Self::DarkGreen,
            Color::Yellow => Self::Yellow,
            Color::DarkYellow => Self::DarkYellow,
            Color::Blue => Self::Blue,
            Color::DarkBlue => Self::DarkBlue,
            Color::Magenta => Self::Magenta,
            Color::DarkMagenta => Self::DarkMagenta,
            Color::Cyan => Self::Cyan,
            Color::DarkCyan => Self::DarkCyan,
            Color::White => Self::White,
            Color::Grey => Self::Grey,
        }
    }
}

#[cfg(feature = "cli")]
impl From<crossterm::style::Color> for Color {
    fn from(c: crossterm::style::Color) -> Self {
        match c {
            crossterm::style::Color::Reset => Self::Reset,
            crossterm::style::Color::Black => Self::Black,
            crossterm::style::Color::DarkGrey => Self::DarkGrey,
            crossterm::style::Color::Red => Self::Red,
            crossterm::style::Color::DarkRed => Self::DarkRed,
            crossterm::style::Color::Green => Self::Green,
            crossterm::style::Color::DarkGreen => Self::DarkGreen,
            crossterm::style::Color::Yellow => Self::Yellow,
            crossterm::style::Color::DarkYellow => Self::DarkYellow,
            crossterm::style::Color::Blue => Self::Blue,
            crossterm::style::Color::DarkBlue => Self::DarkBlue,
            crossterm::style::Color::Magenta => Self::Magenta,
            crossterm::style::Color::DarkMagenta => Self::DarkMagenta,
            crossterm::style::Color::Cyan => Self::Cyan,
            crossterm::style::Color::DarkCyan => Self::DarkCyan,
            crossterm::style::Color::White => Self::White,
            crossterm::style::Color::Grey => Self::Grey,
            _ => Self::White,
        }
    }
}

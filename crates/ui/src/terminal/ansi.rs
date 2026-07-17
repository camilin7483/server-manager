#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: AnsiColor,
    pub bg: AnsiColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: AnsiColor::Default,
            bg: AnsiColor::Default,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            inverse: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Default,
    Named(NamedColor),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedColor {
    Black = 0,   Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow,
    BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
}

impl NamedColor {
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Black   => (0,   0,   0  ),
            Self::Red     => (205, 0,   0  ),
            Self::Green   => (0,   205, 0  ),
            Self::Yellow  => (205, 205, 0  ),
            Self::Blue    => (0,   0,   238),
            Self::Magenta => (205, 0,   205),
            Self::Cyan    => (0,   205, 205),
            Self::White   => (229, 229, 229),
            Self::BrightBlack   => (127, 127, 127),
            Self::BrightRed     => (255, 0,   0  ),
            Self::BrightGreen   => (0,   255, 0  ),
            Self::BrightYellow  => (255, 255, 0  ),
            Self::BrightBlue    => (92,  92,  255),
            Self::BrightMagenta => (255, 0,   255),
            Self::BrightCyan    => (0,   255, 255),
            Self::BrightWhite   => (255, 255, 255),
        }
    }
}

impl AnsiColor {
    pub fn to_rgb24(&self, default: (u8, u8, u8)) -> (u8, u8, u8) {
        match self {
            Self::Default => default,
            Self::Named(n) => n.to_rgb(),
            Self::Indexed(i) => indexed_to_rgb(*i),
            Self::Rgb(r, g, b) => (*r, *g, *b),
        }
    }
}

pub fn indexed_to_rgb(idx: u8) -> (u8, u8, u8) {
    match idx {
        0..=15 => NamedColor::from_idx(idx).to_rgb(),
        16..=231 => {
            let n = idx - 16;
            let r = (n / 36) * 51;
            let g = ((n / 6) % 6) * 51;
            let b = (n % 6) * 51;
            (r as u8, g as u8, b as u8)
        }
        232..=255 => {
            let v = ((idx - 232) * 10 + 8) as u8;
            (v, v, v)
        }
    }
}

impl NamedColor {
    pub(crate) fn from_idx(idx: u8) -> Self {
        match idx {
            0 => Self::Black,  1 => Self::Red,      2 => Self::Green,
            3 => Self::Yellow, 4 => Self::Blue,     5 => Self::Magenta,
            6 => Self::Cyan,   7 => Self::White,    8 => Self::BrightBlack,
            9 => Self::BrightRed,   10 => Self::BrightGreen,
            11 => Self::BrightYellow, 12 => Self::BrightBlue,
            13 => Self::BrightMagenta, 14 => Self::BrightCyan,
            15 => Self::BrightWhite,
            _ => Self::White,
        }
    }
}

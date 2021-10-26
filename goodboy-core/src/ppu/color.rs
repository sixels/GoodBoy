#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    Black,
    DarkGray,
    LightGray,
    White,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub black: Color,
    pub dark_gray: Color,
    pub light_gray: Color,
    pub white: Color,
}

impl ColorScheme {
    pub const fn new(
        black: Color,
        dark_gray: Color,
        light_gray: Color,
        white: Color,
    ) -> ColorScheme {
        ColorScheme {
            black,
            dark_gray,
            light_gray,
            white,
        }
    }

    pub fn get(&self, color_type: ColorType) -> Color {
        match color_type {
            ColorType::Black => self.black,
            ColorType::DarkGray => self.dark_gray,
            ColorType::LightGray => self.light_gray,
            ColorType::White => self.white,
        }
    }

    pub const BLUE: Self = Self::new(
        Color::rgb(0x221e31),
        Color::rgb(0x41485d),
        Color::rgb(0x778e98),
        Color::rgb(0xc5dbd4),
    );

    pub const BLUE_ALT: Self = Self::new(
        Color::rgb(0x622E4C),
        Color::rgb(0x7540e8),
        Color::rgb(0x608fcf),
        Color::rgb(0x8be5ff),
    );

    pub const GREEN: Self = Self::new(
        Color::rgb(0x172808),
        Color::rgb(0x376d03),
        Color::rgb(0x6ab417),
        Color::rgb(0xbeeb71),
    );

    pub const RED: Self = Self::new(
        Color::rgb(0x2c1e74),
        Color::rgb(0xc23a73),
        Color::rgb(0xd58863),
        Color::rgb(0xdad3af),
    );

    pub const GRAY: Self = Self::new(
        Color::rgb(0x211E20),
        Color::rgb(0x555568),
        Color::rgb(0xA0A08B),
        Color::rgb(0xE9EFEC),
    );
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::BLUE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self::rgb(0xFFFFFF);

    pub const fn rgb(color: u32) -> Self {
        let rgb: [u8; 4] = (color << 8).to_be_bytes();

        Color {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 0xFF,
        }
    }

    pub const fn as_slice(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

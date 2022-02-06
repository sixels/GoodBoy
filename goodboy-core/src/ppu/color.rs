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
        Color::new_rgb(0x221e31),
        Color::new_rgb(0x41485d),
        Color::new_rgb(0x778e98),
        Color::new_rgb(0xc5dbd4),
    );

    pub const BLUE_ALT: Self = Self::new(
        Color::new_rgb(0x622E4C),
        Color::new_rgb(0x7540e8),
        Color::new_rgb(0x608fcf),
        Color::new_rgb(0x8be5ff),
    );

    pub const GREEN: Self = Self::new(
        Color::new_rgb(0x172808),
        Color::new_rgb(0x376d03),
        Color::new_rgb(0x6ab417),
        Color::new_rgb(0xbeeb71),
    );

    pub const RED: Self = Self::new(
        Color::new_rgb(0x2c1e74),
        Color::new_rgb(0xc23a73),
        Color::new_rgb(0xd58863),
        Color::new_rgb(0xdad3af),
    );

    pub const GRAY: Self = Self::new(
        Color::new_rgb(0x211E20),
        Color::new_rgb(0x555568),
        Color::new_rgb(0xA0A08B),
        Color::new_rgb(0xE9EFEC),
    );
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::BLUE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb555 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Rgb555 {
    pub const fn into_rgba(self) -> Rgba {
        let Self { r, g, b } = self;

        let (r, g, b) = (r as u32, g as u32, b as u32);

        Rgba {
            r: ((r * 13 + g * 2 + b) >> 1) as u8,
            g: ((g * 3 + b) << 1) as u8,
            b: ((r * 3 + g * 2 + b * 11) >> 1) as u8,
            // r: ((r as u16 * 0xFF) / 0x1F) as u8,
            // g: ((g as u16 * 0xFF) / 0x1F) as u8,
            // b: ((b as u16 * 0xFF) / 0x1F) as u8,
            a: 0xFF,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Rgba(Rgba),
    Rgb555(Rgb555),
}

impl Color {
    pub const WHITE: Self = Self::new_rgba32(0xFFFFFFFF);
    pub const RGB555_WHITE: Rgb555 = Rgb555 {
        r: 0x1F,
        g: 0x1F,
        b: 0x1F,
    };

    pub const fn new_rgba32(rgba: u32) -> Self {
        let rgba: [u8; 4] = (rgba).to_be_bytes();

        Color::Rgba(Rgba {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        })
    }
    pub const fn new_rgb(rgb: u32) -> Self {
        Color::new_rgba32(rgb << 8 | 0xFF)
    }

    pub const fn new_rgb555(rgb555: Rgb555) -> Self {
        Color::Rgb555(rgb555)
    }

    pub const fn into_rgba(self) -> Rgba {
        match self {
            Color::Rgba(rgba) => rgba,
            Color::Rgb555(rgb555) => rgb555.into_rgba(),
        }
    }

    pub const fn into_rgba_slice(self) -> [u8; 4] {
        let rgba = self.into_rgba();
        [rgba.r, rgba.g, rgba.b, rgba.a]
    }
}

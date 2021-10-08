#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0x221e31);
    pub const DGRAY: Color = Color::rgb(0x41485d);
    pub const LGRAY: Color = Color::rgb(0x778e98);
    pub const WHITE: Color = Color::rgb(0xc5dbd4);

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
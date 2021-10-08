#[derive(Debug, Clone, Copy, Default)]
pub struct Sprite {
    pub x: i32,
    pub y: i32,

    pub tile_number: u16,

    pub priority: bool,

    pub flip_x: bool,
    pub flip_y: bool,

    pub palette: bool,
}

pub fn update_sprites(sprites: &mut [Sprite], sprite_addr: usize, sprite_value: u8) {
    let i = sprite_addr >> 2;
    match sprite_addr & 0x03 {
        0 => sprites[i].y = sprite_value as u16 as i32 - 16,
        1 => sprites[i].x = sprite_value as u16 as i32 - 8,
        2 => sprites[i].tile_number = sprite_value as u16,
        _ => {
            sprites[i].priority = (sprite_value & 0x80) == 0x80;
            sprites[i].flip_y = (sprite_value & 0x40) == 0x40;
            sprites[i].flip_x = (sprite_value & 0x20) == 0x20;
            sprites[i].palette = (sprite_value & 0x10) == 0x10;
        }
    }
}

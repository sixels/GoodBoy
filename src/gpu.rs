use crate::{
    memory::MemoryAccess,
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH},
};

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;

#[derive(Debug, Clone, Copy)]
struct Color {
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

bitflags::bitflags! {
    #[derive(Default)]
    struct LCDControl: u8 {
        /// 0 -> Off
        /// 1 -> On
        const LCD_POWER = 1 << 7;
        /// 0 -> 0x9800..=0x9BFF
        /// 1 -> 0x9C00..=0x9FFF
        const WIN_TILEMAP = 1 << 6;
        /// 0 -> Off
        /// 1 -> On
        const WIN_ENABLE = 1 << 5;
        /// 0 -> 0x8800..=0x9CFF
        /// 1 -> 0x8000..=0x8FFF
        const BG_WIN_TILESET = 1 << 4;
        /// 0 -> 0x9800..=0x9BFF
        /// 1 -> 0x9C00..=0x9FFF
        const BG_TILEMAP = 1 << 3;
        /// 0 -> 8x8
        /// 1 -> 8x16
        const SPRITE_SIZE = 1 << 2;
        /// 0 -> Off
        /// 1 -> On
        const SPRITES_ENABLE = 1 << 1;
        /// 0 -> Off
        /// 1 -> On
        const BG_ENABLE = 1 << 0;
    }
}

impl LCDControl {
    #[inline(always)]
    fn lcd_on(&self) -> bool {
        self.contains(Self::LCD_POWER)
    }

    #[inline(always)]
    fn win_on(&self) -> bool {
        self.contains(Self::WIN_ENABLE)
    }

    #[inline(always)]
    fn win_tilemap(&self) -> u16 {
        if !self.contains(Self::WIN_TILEMAP) {
            0x9800
        } else {
            0x9C00
        }
    }

    #[inline(always)]
    fn tileset_base(&self) -> u16 {
        if !self.contains(Self::BG_WIN_TILESET) {
            0x8800
        } else {
            0x8000
        }
    }

    #[inline(always)]
    fn bg_tilemap(&self) -> u16 {
        if !self.contains(Self::BG_TILEMAP) {
            0x9800
        } else {
            0x9C00
        }
    }

    #[inline(always)]
    fn sprite_size(&self) -> u32 {
        if !self.contains(Self::SPRITE_SIZE) {
            8
        } else {
            16
        }
    }

    #[inline(always)]
    fn sprites_on(&self) -> bool {
        self.contains(Self::SPRITES_ENABLE)
    }

    #[inline(always)]
    fn bg_enabled(&self) -> bool {
        self.contains(Self::BG_ENABLE)
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    struct LCDStatus: u8 {
        const SCAN_LINE_CHECK = 1 << 6;
        const OAM_CHECK = 1 << 5;
        const VBLANK_CHECK = 1 << 4;
        const HBLANK_CHECK = 1 << 3;
    }
}

impl LCDStatus {
    fn scanline_check(&self) -> bool {
        self.contains(Self::SCAN_LINE_CHECK)
    }
    fn oam_check(&self) -> bool {
        self.contains(Self::OAM_CHECK)
    }
    fn vblank_check(&self) -> bool {
        self.contains(Self::VBLANK_CHECK)
    }
    fn hblank_check(&self) -> bool {
        self.contains(Self::HBLANK_CHECK)
    }
}

#[derive(PartialEq, PartialOrd)]
enum GpuMode {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

#[derive(Debug, Clone, Copy, Default)]
struct Sprite {
    x: i32,
    y: i32,

    tile_number: u16,

    priority: bool,

    flip_x: bool,
    flip_y: bool,

    palette: bool,
}

pub struct Gpu {
    vram: Box<[u8; VRAM_SIZE]>,
    oam: [u8; OAM_SIZE],
    pub screen_buffer: Screen,

    lcd_control: LCDControl,
    lcd_status: LCDStatus,
    mode: GpuMode,

    scan_line: u8,
    scan_line_check: u8,

    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,

    bg_palette: u8,
    object_palette: [u8; 2],
    palettes: [[Color; 4]; 3],

    bg_priorities: [u8; SCREEN_WIDTH],

    sprites: [Sprite; 40],

    pub interrupt_vblank: bool,
    pub interrupt_lcd: bool,
    clocks: u32,

    pub vblanked: bool,
}

impl Gpu {
    pub fn sync(&mut self, clocks: u32) {
        // Check if lcd is off
        if !self.lcd_control.lcd_on() {
            return;
        }

        let mut clocks = clocks;

        while clocks > 0 {
            let current_clock = if clocks >= 80 { 80 } else { clocks };

            self.clocks += current_clock as u32;
            clocks -= current_clock;

            if self.clocks >= 456 {
                self.clocks -= 456;

                self.scan_line = (self.scan_line + 1) % 154;

                self.interrupt_lyc();

                if self.scan_line >= 144 && self.mode != GpuMode::VBlank {
                    self.update_mode(GpuMode::VBlank)
                }
            }

            if self.scan_line < 144 {
                if self.clocks <= 80 && self.mode != GpuMode::OAMSearch {
                    self.update_mode(GpuMode::OAMSearch)
                } else if self.clocks <= (80 + 172) && self.mode != GpuMode::PixelTransfer {
                    self.update_mode(GpuMode::PixelTransfer)
                } else if self.mode != GpuMode::HBlank {
                    self.update_mode(GpuMode::HBlank)
                }
            }
        }
    }

    fn interrupt_lyc(&mut self) {
        if self.lcd_status.scanline_check() && self.scan_line_check == self.scan_line {
            self.interrupt_lcd = true
        }
    }

    fn update_mode(&mut self, mode: GpuMode) {
        self.mode = mode;

        if match self.mode {
            GpuMode::HBlank => {
                self.draw_scanline();
                self.lcd_status.hblank_check()
            }
            GpuMode::VBlank => {
                self.interrupt_vblank = true;
                self.vblanked = true;

                self.lcd_status.vblank_check()
            }
            GpuMode::OAMSearch => self.lcd_status.oam_check(),
            _ => false,
        } {
            self.interrupt_lcd = true
        }
    }

    fn draw_scanline(&mut self) {
        for x in 0..SCREEN_WIDTH {
            self.set_color(x, Color::WHITE);
            self.bg_priorities[x] = 2;
        }

        self.render_bg();
        self.render_sprites();
    }

    fn render_bg(&mut self) {
        let win_y = if !self.lcd_control.win_on() {
            -1
        } else {
            self.scan_line as i32 - self.win_y as i32
        };

        // No window nor background to be drawn
        if win_y < 0 && !self.lcd_control.bg_enabled() {
            return;
        }

        let win_tile_y = ((win_y as u16) >> 3) & 31;

        let bg_y = self.scroll_y.wrapping_add(self.scan_line);
        let bg_tile_y = ((bg_y as u16) >> 3) & 31;

        for x in 0..SCREEN_WIDTH {
            let win_x = -((self.win_x as i32) - 7) + (x as i32);
            let bg_x = self.scroll_x as u32 + x as u32;

            let (tilemap_base, tile_y, tile_x, pixel_y, pixel_x);

            if win_y >= 0 && win_x >= 0 {
                tilemap_base = self.lcd_control.win_tilemap();
                tile_y = win_tile_y;
                tile_x = (win_x as u16) >> 3;
                pixel_y = win_y as u16 & 0x07;
                pixel_x = win_x as u8 & 0x07;
            } else if self.lcd_control.bg_enabled() {
                tilemap_base = self.lcd_control.bg_tilemap();
                tile_y = bg_tile_y;
                tile_x = ((bg_x as u16) >> 3) & 31;
                pixel_y = bg_y as u16 & 0x07;
                pixel_x = bg_x as u8 & 0x07;
            } else {
                continue;
            }

            let tile_nr = self.mem_read(tilemap_base + tile_y * 32 + tile_x);

            let tile_addr = self.lcd_control.tileset_base()
                + (if self.lcd_control.tileset_base() == 0x8000 {
                    tile_nr as u16
                } else {
                    (tile_nr as i8 as i16 + 128) as u16
                }) * 16;

            let a0 = tile_addr + pixel_y * 2;
            let (b1, b2) = (self.mem_read(a0), self.mem_read(a0 + 1));

            let xbit = 7 - pixel_x;
            let colornr = (if b1 & (1 << xbit) != 0 { 1 } else { 0 })
                | (if b2 & (1 << xbit) != 0 { 2 } else { 0 });

            self.bg_priorities[x] = if colornr == 0 { 0 } else { 2 };

            let color = self.palettes[0][colornr as usize];

            self.set_color(x, color);
        }
    }

    fn render_sprites(&mut self) {
        if !self.lcd_control.sprites_on() {
            return;
        }

        for sprite_nr in 0..40 {
            let i = 39 - sprite_nr;

            // let sprite = self.sprites[i];
            let mut sprite = Sprite::default();
            let sprite_addr = 0xFE00 + (i as u16) * 4;

            sprite.y = self.mem_read(sprite_addr + 0) as u16 as i32 - 16;
            sprite.x = self.mem_read(sprite_addr + 1) as u16 as i32 - 8;

            sprite.tile_number = (self.mem_read(sprite_addr + 2)
                & if self.lcd_control.sprite_size() == 16 {
                    0xFE
                } else {
                    0xFF
                }) as u16;

            let flags = self.mem_read(sprite_addr + 3) as usize;
            sprite.palette = flags & (1 << 4) != 0;
            sprite.flip_x = flags & (1 << 5) != 0;
            sprite.flip_y = flags & (1 << 6) != 0;
            sprite.priority = flags & (1 << 7) != 0;

            let scan_line = self.scan_line as i32;
            let sprite_size = self.lcd_control.sprite_size() as i32;

            // Check if the sprite is in the screen
            if (scan_line < sprite.y || scan_line >= (sprite.y + sprite_size))
                || (sprite.x < -7 || sprite.x >= SCREEN_WIDTH as i32)
            {
                continue;
            }

            let tile_y = if sprite.flip_y {
                sprite_size - 1 - (scan_line - sprite.y)
            } else {
                scan_line - sprite.y
            } as u16;

            let tile_nr = sprite.tile_number & (if sprite_size == 16 { 0xFE } else { 0xFF });
            let tile_addr = 0x8000u16 + tile_nr * 16 + tile_y * 2;

            let (b1, b2) = (self.mem_read(tile_addr), self.mem_read(tile_addr + 1));

            'bits: for x in 0..8 {
                if sprite.x + x < 0 || sprite.x + x >= (SCREEN_WIDTH as i32) {
                    continue;
                }

                let xbit = 1 << (if sprite.flip_x { x } else { 7 - x });
                let color_nr =
                    (if b1 & xbit != 0 { 1 } else { 0 }) | (if b2 & xbit != 0 { 2 } else { 0 });

                // the pixel is transparent
                if color_nr == 0 {
                    continue;
                }

                if sprite.priority && self.bg_priorities[(sprite.x + x) as usize] != 0 {
                    continue 'bits;
                }

                let color = self.palettes[((sprite.palette ^ true) as u8 + 1) as usize][color_nr];
                self.set_color((sprite.x + x) as usize, color)
            }
        }
    }

    fn set_color(&mut self, x: usize, color: Color) {
        let y = self.scan_line as usize;
        for (i, rgb) in color.as_slice().iter().enumerate() {
            self.screen_buffer[y * SCREEN_WIDTH * 4 + x * 4 + i] = *rgb;
        }
    }

    fn update_palettes(&mut self, palette_nr: usize) {
        debug_assert!(palette_nr <= 3);

        for i in 0..4 {
            let color = match i {
                0 => Color::WHITE,
                1 => Color::LGRAY,
                2 => Color::DGRAY,
                3 => Color::BLACK,
                _ => unreachable!(),
            };

            self.palettes[palette_nr][i] = color;
        }
    }

    fn update_sprite(&mut self, sprite_addr: u16) {
        let sprite_addr = sprite_addr as usize;

        let i = sprite_addr >> 2;
        let value = self.oam[i];

        match sprite_addr & 0x03 {
            0 => self.sprites[i].y = value as u16 as i32 - 16,
            1 => self.sprites[i].x = value as u16 as i32 - 8,
            2 => self.sprites[i].tile_number = value as u16,
            3 => {
                self.sprites[i].priority = (value & 0x80) == 0x80;
                self.sprites[i].flip_y = (value & 0x40) == 0x40;
                self.sprites[i].flip_x = (value & 0x20) == 0x20;
                self.sprites[i].palette = (value & 0x10) == 0x10;
            }
            _ => (),
        }
    }

    fn clear_screen(&mut self) {
        for pixels in self.screen_buffer.chunks_mut(4) {
            pixels.copy_from_slice(&Color::WHITE.as_slice());
        }
        self.vblanked = true;
    }
}

impl MemoryAccess for Gpu {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr & 0x7FFF) as usize],
            0xFE00..=0xFE9F => self.oam[(addr & 0x01FF) as usize],

            0xFF40 => self.lcd_control.bits(),
            0xFF41 => self.lcd_status.bits(),

            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,

            0xFF44 => self.scan_line,
            0xFF45 => self.scan_line_check,

            0xFF47 => self.bg_palette,
            0xFF48..=0xFF49 => self.object_palette[(addr & 1) as usize],

            0xFF4A => self.win_y,
            0xFF4B => self.win_x,

            _ => panic!("Invalid GPU read address: 0x{:04X}", addr),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr & 0x7FFF) as usize] = value,
            0xFE00..=0xFE9F => {
                let addr = addr & 0x01FF;
                self.oam[addr as usize] = value;
                self.update_sprite(addr);
            }

            0xFF40 => {
                let lcd_state = self.lcd_control.lcd_on();
                self.lcd_control = LCDControl::from_bits_truncate(value);
                let new_lcd_state = self.lcd_control.lcd_on();
                if lcd_state && !new_lcd_state {
                    self.mode = GpuMode::HBlank;
                    self.clocks = 0;
                    self.scan_line = 0;
                    self.clear_screen();
                }
            }
            0xFF41 => self.lcd_status = LCDStatus::from_bits_truncate(value),

            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,

            // 0xFF44 => READ-ONLY
            0xFF45 => self.scan_line_check = value,

            0xFF47 => {
                self.bg_palette = value;
                self.update_palettes(0);
            }
            0xFF48..=0xFF49 => {
                let palette_nr = (addr & 1) as usize;
                self.object_palette[palette_nr] = value;
                self.update_palettes(palette_nr + 1);
            }

            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,

            _ => panic!("Invalid GPU write address: 0x{:04X}", addr),
        }
    }
}

impl Default for Gpu {
    fn default() -> Gpu {
        Gpu {
            vram: box [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            screen_buffer: box [0x00; SCREEN_WIDTH * SCREEN_HEIGHT * 4],

            lcd_control: LCDControl::default(),
            lcd_status: LCDStatus::default(),
            mode: GpuMode::HBlank,

            scan_line: 0,
            scan_line_check: 0,

            scroll_x: 0,
            scroll_y: 0,
            win_x: 0,
            win_y: 0,

            bg_palette: 0,
            object_palette: [0, 0],
            palettes: [[Color::WHITE; 4]; 3],

            bg_priorities: [0; SCREEN_WIDTH],
            sprites: [Default::default(); 40],

            interrupt_vblank: false,
            interrupt_lcd: false,
            clocks: 0,
            vblanked: false,
        }
    }
}

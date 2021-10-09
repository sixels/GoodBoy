use crate::{
    mmu::MemoryAccess,
    ppu::{
        color::ColorType,
        lcd::{LCDControl, LCDStatus},
        palette::{PaletteKind, Palettes},
        sprites, Color, Sprite,
    },
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH},
};

use super::color::ColorScheme;

const OAM_SIZE: usize = 0xA0;
const VRAM_SIZE: usize = 0x4000;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum GpuMode {
    HBlank,
    VBlank,
    OAMSearch,
    PixelTransfer,
}

impl From<GpuMode> for u8 {
    fn from(mode: GpuMode) -> Self {
        match mode {
            GpuMode::VBlank => 1,
            GpuMode::OAMSearch => 2,
            GpuMode::PixelTransfer => 3,
            _ => 0,
        }
    }
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
    palettes: Palettes,

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
                self.render_scan_line();
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

    fn render_scan_line(&mut self) {
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

        let win_tile_y = (win_y as u16 >> 3) & 31;

        let bg_y = self.scroll_y.wrapping_add(self.scan_line);
        let bg_tile_y = (bg_y as u16 >> 3) & 31;

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
                tile_x = (bg_x as u16 >> 3) & 31;
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

            let xbit = 7 - pixel_x as u32;
            let color_nr = if b1 & (1 << xbit) != 0 { 1 } else { 0 }
                | if b2 & (1 << xbit) != 0 { 2 } else { 0 };

            self.bg_priorities[x] = if color_nr == 0 { 0 } else { 2 };

            let color = self.palettes.get(PaletteKind::BG)[color_nr].0;
            self.set_color(x, color);
        }
    }

    fn render_sprites(&mut self) {
        if !self.lcd_control.sprites_on() {
            return;
        }

        let mut sprites_in_row: usize = 0;

        for sprite in self.sprites {
            // each row can only have 10 sprites
            if sprites_in_row >= 10 {
                break;
            }

            let scan_line = self.scan_line as i32;
            let sprite_size = self.lcd_control.sprite_size() as i32;

            // Check if the sprite is in the screen
            if (scan_line < sprite.y || scan_line >= (sprite.y + sprite_size))
                || (sprite.x < -7 || sprite.x >= SCREEN_WIDTH as i32)
            {
                continue;
            }
            sprites_in_row += 1;

            let tile_y = if sprite.flip_y {
                sprite_size - 1 - (scan_line - sprite.y)
            } else {
                scan_line - sprite.y
            } as u16;

            let tile_number = sprite.tile_number & if sprite_size == 16 { 0xFE } else { 0xFF };
            let tile_addr = 0x8000u16 + tile_number * 16 + tile_y * 2;

            let (b1, b2) = (self.mem_read(tile_addr), self.mem_read(tile_addr + 1));

            'bits: for x in 0..8 {
                if sprite.x + x < 0 || sprite.x + x >= (SCREEN_WIDTH as i32) {
                    continue;
                }

                let xbit = 1 << (if sprite.flip_x { x } else { 7 - x });
                let color_nr =
                    if b1 & xbit != 0 { 1 } else { 0 } | if b2 & xbit != 0 { 2 } else { 0 };

                // the pixel is transparent
                if color_nr == 0 {
                    continue;
                }

                if sprite.priority && self.bg_priorities[(sprite.x + x) as usize] != 0 {
                    continue 'bits;
                }

                let palette = if !sprite.palette {
                    PaletteKind::OBJ0
                } else {
                    PaletteKind::OBJ1
                };
                let color = self.palettes.get(palette)[color_nr].0;
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

    fn update_palette(&mut self, palette: PaletteKind, value: u8) {
        fn get_palette_color(value: u8, i: usize) -> ColorType {
            match (value >> 2 * i) & 0x03 {
                0 => ColorType::White,
                1 => ColorType::LightGray,
                2 => ColorType::DarkGray,
                _ => ColorType::Black,
            }
        }

        for i in 0..4 {
            self.palettes
                .update(palette, i, get_palette_color(value, i));
        }
    }

    fn clear_screen(&mut self) {
        for pixels in self.screen_buffer.chunks_mut(4) {
            pixels.copy_from_slice(&Color::WHITE.as_slice());
        }
        self.vblanked = true;
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.palettes.set_color_scheme(color_scheme)
    }
}

impl MemoryAccess for Gpu {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr & 0x1FFF) as usize],
            0xFE00..=0xFE9F => self.oam[(addr & 0x01FF) as usize],

            0xFF40 => self.lcd_control.bits(),
            0xFF41 => {
                self.lcd_status.bits()
                    | (if self.scan_line == self.scan_line_check {
                        0x04
                    } else {
                        0
                    })
                    | u8::from(self.mode)
            }

            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,

            0xFF44 => self.scan_line,
            0xFF45 => self.scan_line_check,

            0xFF47 => self.bg_palette,
            0xFF48 => self.object_palette[0],
            0xFF49 => self.object_palette[1],

            0xFF4A => self.win_y,
            0xFF4B => self.win_x,

            _ => panic!("Invalid GPU read address: 0x{:04X}", addr),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr & 0x1FFF) as usize] = value,
            0xFE00..=0xFE9F => {
                let addr = addr as usize & 0x01FF;
                self.oam[addr] = value;

                sprites::update_sprites(&mut self.sprites, addr, value);
            }

            0xFF40 => {
                let lcd_was_enable = self.lcd_control.lcd_on();
                self.lcd_control = LCDControl::from_bits_truncate(value);
                if lcd_was_enable && !self.lcd_control.lcd_on() {
                    self.mode = GpuMode::HBlank;
                    self.clocks = 0;
                    self.scan_line = 0;
                    self.clear_screen();
                }
            }
            0xFF41 => self.lcd_status = LCDStatus::from_bits_truncate(value),

            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,

            0xFF44 => (), // READ-ONLY
            0xFF45 => self.scan_line_check = value,

            0xFF47 => {
                self.bg_palette = value;
                self.update_palette(PaletteKind::BG, value);
            }
            0xFF48 => {
                self.object_palette[0] = value;
                self.update_palette(PaletteKind::OBJ0, value);
            }
            0xFF49 => {
                self.object_palette[1] = value;
                self.update_palette(PaletteKind::OBJ1, value);
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
            palettes: Palettes::default(),

            bg_priorities: [0; SCREEN_WIDTH],
            sprites: [Default::default(); 40],

            interrupt_vblank: false,
            interrupt_lcd: false,
            clocks: 0,
            vblanked: false,
        }
    }
}

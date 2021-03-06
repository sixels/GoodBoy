use crate::{
    gb_mode::GbMode,
    mmu::MemoryAccess,
    ppu::{
        color::ColorType,
        lcd::{LCDControl, LCDStatus},
        palette::{PaletteKind, Palettes},
        sprites, Color, Sprite,
    },
    vm::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH},
};

use super::color::{ColorScheme, Rgb555};

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
            GpuMode::HBlank => 0,
            GpuMode::VBlank => 1,
            GpuMode::OAMSearch => 2,
            GpuMode::PixelTransfer => 3,
        }
    }
}

pub struct Gpu {
    gb_mode: GbMode,

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
    pub interrupt: u8,
    clocks: u32,

    pub vblanked: bool,

    // CGB registers
    vram_bank: usize,
    cgb_bgpal_auto_inc: bool,
    cgb_bgpal_addr: u8,
    // CGB stores color as RGB555, we need to convert it to RGBA later
    cgb_bgpal: [[Rgb555; 4]; 8], // 8 palettes with 4 colors per palette

    cgb_sppal_auto_inc: bool,
    cgb_sppal_addr: u8,
    cgb_sppal: [[Rgb555; 4]; 8],

    pub hblanking: bool,
}

impl Gpu {
    pub fn new(gb_mode: GbMode) -> Self {
        Gpu {
            gb_mode,

            vram: Box::new([0; VRAM_SIZE]),
            oam: [0; OAM_SIZE],
            screen_buffer: Box::new([0x00; SCREEN_WIDTH * SCREEN_HEIGHT * 4]),

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
            interrupt: 0,
            clocks: 0,
            vblanked: false,

            vram_bank: 0,
            cgb_bgpal_auto_inc: false,
            cgb_bgpal_addr: 0,
            cgb_bgpal: [[Color::RGB555_WHITE; 4]; 8],

            cgb_sppal_auto_inc: false,
            cgb_sppal_addr: 0,
            cgb_sppal: [[Color::RGB555_WHITE; 4]; 8],

            hblanking: false,
        }
    }

    pub fn sync(&mut self, clocks: u32) {
        // Check if lcd is off
        if !self.lcd_control.lcd_on() {
            return;
        }
        self.hblanking = false;

        let mut clocks = clocks;

        while clocks > 0 {
            let ran_clocks = if clocks >= 80 { 80 } else { clocks };

            self.clocks += ran_clocks;
            clocks -= ran_clocks;

            if self.clocks >= 456 {
                self.clocks -= 456;
                self.scan_line = (self.scan_line + 1) % 154;
                self.interrupt_lyc();

                if self.scan_line >= 144 && self.mode != GpuMode::VBlank {
                    self.change_mode(GpuMode::VBlank);
                }
            }

            if self.scan_line < 144 {
                if self.clocks <= 80 {
                    if self.mode != GpuMode::OAMSearch {
                        self.change_mode(GpuMode::OAMSearch);
                    }
                } else if self.clocks <= (80 + 172) {
                    if self.mode != GpuMode::PixelTransfer {
                        self.change_mode(GpuMode::PixelTransfer);
                    }
                } else {
                    if self.mode != GpuMode::HBlank {
                        self.change_mode(GpuMode::HBlank);
                    }
                }
            }
        }
    }

    fn change_mode(&mut self, mode: GpuMode) {
        self.mode = mode;

        if match self.mode {
            GpuMode::HBlank => {
                self.render_scan_line();
                self.hblanking = true;
                self.lcd_status.hblank_check()
            }
            GpuMode::VBlank => {
                self.interrupt |= 0x01;
                self.vblanked = true;
                self.lcd_status.vblank_check()
            }
            GpuMode::OAMSearch => self.lcd_status.oam_check(),
            GpuMode::PixelTransfer => false,
        } {
            self.interrupt |= 0x02;
        }
    }

    fn interrupt_lyc(&mut self) {
        if self.lcd_status.scanline_check() && self.scan_line_check == self.scan_line {
            self.interrupt |= 0x02;
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
        let do_render = self.gb_mode == GbMode::Cgb || self.lcd_control.bg_enabled();

        let win_y = if !self.lcd_control.win_on()
            || (self.gb_mode != GbMode::Dmg && !self.lcd_control.bg_enabled())
        {
            -1
        } else {
            self.scan_line as i32 - self.win_y as i32
        };

        // No window nor background to be drawn
        if win_y < 0 && !do_render {
            return;
        }

        let win_tile_y = (win_y as u16 >> 3) & 31;

        let bg_y = self.scroll_y.wrapping_add(self.scan_line);
        let bg_tile_y = (bg_y as u16 >> 3) & 31;

        for x in 0..SCREEN_WIDTH {
            let win_x = -((self.win_x as i32) - 7) + (x as i32);
            let bg_x = self.scroll_x as u32 + x as u32;

            let (tilemap_base, tile_y, tile_x, pixel_y, pixel_x) = if win_y >= 0 && win_x >= 0 {
                (
                    self.lcd_control.win_tilemap(),
                    win_tile_y,
                    (win_x as u16 >> 3),
                    win_y as u16 & 0x07,
                    win_x as u8 & 0x07,
                )
            } else if do_render {
                (
                    self.lcd_control.bg_tilemap(),
                    bg_tile_y,
                    (bg_x as u16 >> 3) & 31,
                    bg_y as u16 & 0x07,
                    bg_x as u8 & 0x07,
                )
            } else {
                continue;
            };

            let tile_nr = self.mem_read(tilemap_base + tile_y * 32 + tile_x);

            let (paletten, use_bank1, xflip, yflip, prio) = if self.gb_mode == GbMode::Cgb {
                let f = self.read_bank1(tilemap_base + tile_y * 32 + tile_x) as usize;
                (
                    f & 0b111,
                    (f & 0x08) == 0x08,
                    (f & 0x20) == 0x20,
                    (f & 0x40) == 0x40,
                    (f & 0x80) == 0x80,
                )
            } else {
                (0, false, false, false, false)
            };

            let tile_addr = self.lcd_control.tileset_base()
                + (if self.lcd_control.tileset_base() == 0x8000 {
                    tile_nr as u16
                } else {
                    (tile_nr as i8 as i16 + 128) as u16
                }) * 16;

            let a0 = if yflip {
                tile_addr + 14 - pixel_y * 2
            } else {
                tile_addr + pixel_y * 2
            };

            let (b1, b2) = if use_bank1 {
                (self.read_bank1(a0), self.read_bank1(a0 + 1))
            } else {
                (self.read_bank0(a0), self.read_bank0(a0 + 1))
            };

            let xbit = if xflip { pixel_x } else { 7 - pixel_x } as u32;
            let colorn = if b1 & (1 << xbit) != 0 { 1 } else { 0 }
                | if b2 & (1 << xbit) != 0 { 2 } else { 0 };

            self.bg_priorities[x] = if colorn == 0 {
                0
            } else if prio {
                1
            } else {
                2
            };

            let color = if self.gb_mode != GbMode::Dmg {
                Color::new_rgb555(self.cgb_bgpal[paletten][colorn])
            } else {
                self.palettes.get(PaletteKind::BG)[colorn].0
            };
            self.set_color(x, color);
        }
    }

    fn render_sprites(&mut self) {
        if !self.lcd_control.sprites_on() {
            return;
        }

        let mut sprites_in_row: usize = 0;
        let sprites = self.sprites.clone();
        for sprite in sprites.iter().rev() {
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
            // sprites_in_row += 1;

            let tile_y = if sprite.flip_y {
                sprite_size - 1 - (scan_line - sprite.y)
            } else {
                scan_line - sprite.y
            } as u16;

            let tile_number = sprite.tile_number & if sprite_size == 16 { 0xFE } else { 0xFF };
            let tile_addr = 0x8000 + tile_number * 16 + tile_y * 2;

            let (b1, b2) = if sprite.vram_bank == 1 && self.gb_mode == GbMode::Cgb {
                (self.read_bank1(tile_addr), self.read_bank1(tile_addr + 1))
            } else {
                (self.read_bank0(tile_addr), self.read_bank0(tile_addr + 1))
            };

            'bits: for x in 0..8 {
                if sprite.x + x < 0 || sprite.x + x >= (SCREEN_WIDTH as i32) {
                    continue;
                }

                let xbit = 1 << (if sprite.flip_x { x } else { 7 - x });
                let colorn =
                    if b1 & xbit != 0 { 1 } else { 0 } | if b2 & xbit != 0 { 2 } else { 0 };

                // the pixel is transparent
                if colorn == 0 {
                    continue;
                }

                let xpriority = self.bg_priorities[(sprite.x + x) as usize];
                let bg_is_priority = (self.gb_mode == GbMode::Cgb && xpriority == 1)
                    || (sprite.priority && xpriority != 0);

                let color = if self.gb_mode == GbMode::Cgb {
                    if self.lcd_control.bg_enabled() && bg_is_priority {
                        continue 'bits;
                    }
                    Color::new_rgb555(self.cgb_sppal[sprite.paletten][colorn])
                } else {
                    if bg_is_priority {
                        continue 'bits;
                    }

                    let palette = if !sprite.palette {
                        PaletteKind::OBJ0
                    } else {
                        PaletteKind::OBJ1
                    };
                    self.palettes.get(palette)[colorn].0
                };
                self.set_color((sprite.x + x) as usize, color)
            }
        }
    }

    fn read_bank0(&self, addr: u16) -> u8 {
        if addr < 0x8000 || addr >= 0xA000 {
            panic!("Reading a non-bank0 address: 0x{addr:04x}");
        }
        self.vram[addr as usize & 0x1FFF]
    }
    fn read_bank1(&self, addr: u16) -> u8 {
        if addr < 0x8000 || addr >= 0xA000 {
            panic!("Reading a non-bank1 address: 0x{addr:04x}");
        }
        self.vram[0x2000 + (addr as usize & 0x1FFF)]
    }

    fn set_color(&mut self, x: usize, color: Color) {
        let y = self.scan_line as usize;
        for (i, rgba) in color.into_rgba_slice().iter().enumerate() {
            self.screen_buffer[y * SCREEN_WIDTH * 4 + x * 4 + i] = *rgba;
        }
    }

    fn update_palette(&mut self, palette: PaletteKind, value: u8) {
        fn get_palette_color(value: u8, i: usize) -> ColorType {
            match (value >> (i << 1)) & 0x03 {
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
            pixels.copy_from_slice(&Color::WHITE.into_rgba_slice());
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
            0x8000..=0x9FFF => self.vram[(self.vram_bank * 0x2000) | (addr & 0x1FFF) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],

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

            0xff46 => 0,

            0xFF47 => self.bg_palette,
            0xFF48 => self.object_palette[0],
            0xFF49 => self.object_palette[1],

            0xFF4A => self.win_y,
            0xFF4B => self.win_x,

            0xff4f => self.vram_bank as u8,

            0xff68 => self.cgb_bgpal_addr | if self.cgb_bgpal_auto_inc { 0x80 } else { 0 },
            0xff69 => {
                let paletten = (self.cgb_bgpal_addr >> 3) as usize;
                let colorn = ((self.cgb_bgpal_addr >> 1) & 0x03) as usize;

                let color = self.cgb_bgpal[paletten][colorn];
                if self.cgb_bgpal_addr & 1 == 0 {
                    // even
                    color.r | ((color.g & 0x07) << 5)
                } else {
                    // odd
                    ((color.g & 0x18) >> 3) | (color.b << 2)
                }
            }
            0xff6a => self.cgb_sppal_addr | if self.cgb_sppal_auto_inc { 0x80 } else { 0 },
            0xff6b => {
                let paletten = (self.cgb_sppal_addr >> 3) as usize;
                let colorn = 0x03 & (self.cgb_sppal_addr >> 1) as usize;

                let color = self.cgb_sppal[paletten][colorn];
                if self.cgb_sppal_addr & 1 == 0 {
                    // even
                    color.r | ((color.g & 0x07) << 5)
                } else {
                    // odd
                    ((color.g & 0x18) >> 3) | (color.b << 2)
                }
            }
            _ => panic!("Invalid GPU read address: 0x{:04X}", addr),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => {
                self.vram[(self.vram_bank * 0x2000) | (addr & 0x1FFF) as usize] = value
            }
            0xFE00..=0xFE9F => {
                let addr = (addr - 0xFE00) as usize;
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

            0xff4f => self.vram_bank = 0x01 & value as usize,

            0xff68 => {
                self.cgb_bgpal_addr = value & 0x3f;
                self.cgb_bgpal_auto_inc = value & 0x80 == 0x80;
            }
            0xff69 => {
                let paletten = (self.cgb_bgpal_addr >> 3) as usize;
                let colorn = 0x03 & (self.cgb_bgpal_addr >> 1) as usize;

                let color = &mut self.cgb_bgpal[paletten][colorn];

                if self.cgb_bgpal_addr & 1 == 0 {
                    color.r = value & 0x1F;
                    color.g = (color.g & 0x18) | (value >> 5);
                } else {
                    color.g = (color.g & 0x07) | ((value & 0x03) << 3);
                    color.b = (value >> 2) & 0x1F;
                }

                self.cgb_bgpal_auto_inc
                    .then(|| self.cgb_bgpal_addr = (self.cgb_bgpal_addr + 1) & 0x3F);
            }
            0xff6a => {
                self.cgb_sppal_addr = value & 0x3f;
                self.cgb_sppal_auto_inc = value >> 7 == 1;
            }
            0xff6b => {
                let paletten = (self.cgb_sppal_addr >> 3) as usize;
                let colorn = 0x03 & (self.cgb_sppal_addr >> 1) as usize;

                let color = &mut self.cgb_sppal[paletten][colorn];

                if self.cgb_sppal_addr & 1 == 0 {
                    color.r = value & 0x1F;
                    color.g = (color.g & 0x18) | (value >> 5);
                } else {
                    color.g = (color.g & 0x07) | ((value & 0x03) << 3);
                    color.b = (value >> 2) & 0x1F;
                }

                self.cgb_sppal_auto_inc
                    .then(|| self.cgb_sppal_addr = (self.cgb_sppal_addr + 1) & 0x3F);
            }

            _ => panic!("Invalid GPU write address: 0x{:04X}", addr),
        }
    }
}

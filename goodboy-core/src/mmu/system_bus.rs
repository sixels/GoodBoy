use std::iter;

use crate::{
    gb_mode::GbMode,
    io::{Joypad, Serial, Timer},
    ppu::Gpu,
};

use super::{cartridge::Cartridge, InterruptFlags, Mbc, MemoryAccess};

const WRAM_SIZE: usize = 0x8000;
const ZRAM_SIZE: usize = 0x7F;

/// The System Bus
pub struct Bus {
    pub gb_mode: GbMode,

    /// TODO: Handle it in the Cartridge struct
    ///
    /// Cartridge buffer \
    /// 0x0000 ..= 0x3FFF -> ROM0 \
    /// 0x4000 ..= 0x7FFF -> ROMX
    // rom_buffer: Box<[u8; 0x8000]>,
    /// 0xA000 ..= 0xBFFF
    // sram: Box<[u8; 0x2000]>,

    // Cartridge
    cartridge: Cartridge,

    /// Work RAM \
    /// 0xC000 ..= 0xCFFF -> WRAM0 \
    /// 0xD000 ..= 0xDFFF -> WRAMX \
    /// 0xE000 ..= 0xFDFF -> WRAM ECHO
    wram: Vec<u8>,
    /// Zero-page RAM \
    /// 0xFF80 ..= 0xFFFE
    zram: [u8; ZRAM_SIZE],

    /// GPU
    pub gpu: Gpu,

    pub joypad: Joypad,

    /// Timer \
    /// 0xFF04 -> Divider Register (DIV) \
    /// 0xFF05 -> Counter (TIMA) \
    /// 0xFF06 -> Modulo (TMA) \
    /// 0xFF07 -> Control (TAC)
    timer: Timer,

    /// Serial \
    /// 0xFF01 -> Transfer Data (SD) \
    /// 0xFF02 -> Transfer Control (SC)
    serial: Serial,

    /// Other I/O Registers \
    /// 0xFF00 ..= 0xFF7F
    io_registers: [u8; 0x80],

    /// Interrupt Flag (IF) \
    /// 0xFF0F
    pub iflag: InterruptFlags,
    /// Interrupt Enable (IE) \
    /// 0xFFFF
    pub ienable: InterruptFlags,

    // CGB registers
    /// WRAM Bank
    wram_bank: usize,
}

impl Bus {
    pub fn new(rom: &[u8]) -> Bus {
        let wram = iter::repeat(0).take(WRAM_SIZE).collect();

        let (cartridge, gb_mode) = Cartridge::new(rom);

        let mut bus = Bus {
            gb_mode,

            cartridge,
            wram,

            gpu: Default::default(),
            joypad: Default::default(),
            serial: Default::default(),
            timer: Default::default(),
            io_registers: [0; 0x80],
            zram: [0; ZRAM_SIZE],
            ienable: Default::default(),
            iflag: Default::default(),
            wram_bank: 1,
        };

        // Startup sequence
        bus.initialize();

        bus
    }

    fn initialize(&mut self) {
        self.mem_write(0xFF05, 0x00); // TIMA
        self.mem_write(0xFF06, 0x00); // TMA
        self.mem_write(0xFF07, 0x00); // TAC
        self.mem_write(0xFF10, 0x80); // NR10
        self.mem_write(0xFF11, 0xBF); // NR11
        self.mem_write(0xFF12, 0xF3); // NR12
        self.mem_write(0xFF14, 0xBF); // NR14
        self.mem_write(0xFF16, 0x3F); // NR21
        self.mem_write(0xFF17, 0x00); // NR22
        self.mem_write(0xFF19, 0xBF); // NR24
        self.mem_write(0xFF1A, 0x7F); // NR30
        self.mem_write(0xFF1B, 0xFF); // NR31
        self.mem_write(0xFF1C, 0x9F); // NR32
        self.mem_write(0xFF1E, 0xBF); // NR33
        self.mem_write(0xFF20, 0xFF); // NR41
        self.mem_write(0xFF21, 0x00); // NR42
        self.mem_write(0xFF22, 0x00); // NR43
        self.mem_write(0xFF23, 0xBF); // NR44
        self.mem_write(0xFF24, 0x77); // NR50
        self.mem_write(0xFF25, 0xF3); // NR51
        self.mem_write(0xFF26, 0xF1); // NR52
        self.mem_write(0xFF40, 0x91); // LCDC
        self.mem_write(0xFF42, 0x00); // SCY
        self.mem_write(0xFF43, 0x00); // SCX
        self.mem_write(0xFF45, 0x00); // LYC
        self.mem_write(0xFF47, 0xFC); // BGP
        self.mem_write(0xFF48, 0xFF); // OBP0
        self.mem_write(0xFF49, 0xFF); // OBP1
        self.mem_write(0xFF4A, 0x00); // WY
        self.mem_write(0xFF4B, 0x00); // WX
        self.mem_write(0xFFFF, 0x00); // IE
    }

    // pub fn new_blank(bios: &[u8]) -> Bus {
    //     let wram = iter::repeat(0).take(0x2000).collect();

    //     let mut rom_buffer = box [0; 0x8000];
    //     rom_buffer[..bios.len()].copy_from_slice(bios);

    //     Bus {
    //         rom_buffer,
    //         wram,
    //         ..Default::default()
    //     }
    // }

    /// Ticks the IO devices
    pub fn tick(&mut self, clocks: u32) -> u32 {
        // update the timer
        self.timer.sync(clocks);
        if self.timer.interrupt {
            self.iflag.insert(InterruptFlags::TIMER);
            self.timer.interrupt = false;
        }

        // update the gpu
        self.gpu.sync(clocks);
        if self.gpu.interrupt_vblank {
            self.iflag.insert(InterruptFlags::VBLANK);
            self.gpu.interrupt_vblank = false;
        }
        if self.gpu.interrupt_lcd {
            self.iflag.insert(InterruptFlags::LCD);
            self.gpu.interrupt_lcd = false;
        }

        // update the serial
        if self.serial.interrupt {
            self.iflag.insert(InterruptFlags::SERIAL);
            self.serial.interrupt = false;
        }

        clocks
    }
}

impl MemoryAccess for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.rom_read(addr),

            0x8000..=0x9FFF => self.gpu.mem_read(addr),

            0xA000..=0xBFFF => self.cartridge.ram_read(addr),

            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram[(addr & 0x0FFF) as usize],
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.wram[((addr as usize) & 0x0FFF | (self.wram_bank * 0x1000))]
            }

            0xFE00..=0xFE9F => self.gpu.mem_read(addr),

            0xFEA0..=0xFEFF => 0, // unused

            0xFF00 => self.joypad.read(),

            0xFF0F => self.iflag.bits(),

            0xFF01..=0xFF02 => self.serial.mem_read(addr),
            0xFF04..=0xFF07 => self.timer.mem_read(addr),

            0xFF46 => 0,
            0xFF40..=0xFF4B => self.gpu.mem_read(addr),

            0xFF70 => self.wram_bank as u8,
            0xFF00..=0xFF7F => self.io_registers[(addr & 0xFF) as usize],

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize],

            0xFFFF => self.ienable.bits(),
        }
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.rom_write(addr, value),

            0x8000..=0x9FFF => self.gpu.mem_write(addr, value),

            0xA000..=0xBFFF => self.cartridge.ram_write(addr, value),

            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram[(addr & 0x0FFF) as usize] = value,
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.wram[(addr & 0x0FFF | 0x1000) as usize] = value
            }

            0xFE00..=0xFE9F => self.gpu.mem_write(addr, value),

            0xFEA0..=0xFEFF => (), // unused

            0xFF00 => self.joypad.write(value),

            0xFF0F => self.iflag = InterruptFlags::from_bits_truncate(value),

            0xFF01..=0xFF02 => self.serial.mem_write(addr, value),
            0xFF04..=0xFF07 => self.timer.mem_write(addr, value),

            0xFF46 => {
                let base = (value as u16) << 8;
                for i in 0..0xA0 {
                    let b = self.mem_read(base + i);
                    self.mem_write(0xFE00 + i, b);
                }
            }
            0xFF40..=0xFF4B => self.gpu.mem_write(addr, value),

            0xFF70 => {
                self.wram_bank = if (value & 0x7) == 0 {
                    1
                } else {
                    (value & 0x7) as usize
                }
            }
            0xFF00..=0xFF7F => self.io_registers[(addr & 0xFF) as usize] = value,

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize] = value,

            0xFFFF => self.ienable = InterruptFlags::from_bits_truncate(value),
        }
    }
}

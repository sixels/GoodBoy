#![allow(dead_code)]

use std::iter;

use crate::{
    gpu::Gpu,
    io::{Serial, Timer},
    memory::MemoryAccess,
};

bitflags::bitflags! {
    #[derive(Default)]
    pub struct InterruptFlags: u8 {
        const JOYPAD = 1 << 4;
        const SERIAL = 1 << 3;
        const TIMER  = 1 << 2;
        const LCD    = 1 << 1;
        const VBLANK = 1 << 0;
    }
}

/// The System Bus
pub struct Bus {
    /// TODO: Handle it in the Cartridge struct
    ///
    /// Cartridge buffer \
    /// 0x0000 ..= 0x3FFF -> ROM0 \
    /// 0x4000 ..= 0x7FFF -> ROMX
    rom_buffer: Box<[u8; 0x8000]>,
    /// 0xA000 ..= 0xBFFF
    sram: Box<[u8; 0x2000]>,

    /// TODO: Wrap both in the Gpu struct
    ///
    /// Video RAM \
    /// 0x8000 ..= 0x9FFF
    vram: Box<[u8; 0x2000]>,
    /// Sprite information table \
    /// 0xFE00 ..= 0xFE9F
    oam: [u8; 0xA0],

    /// Work RAM \
    /// 0xC000 ..= 0xCFFF -> WRAM0 \
    /// 0xD000 ..= 0xDFFF -> WRAMX \
    /// 0xE000 ..= 0xFDFF -> WRAM ECHO
    wram: Vec<u8>,
    /// Zero-page RAM \
    /// 0xFF80 ..= 0xFFFE
    zram: [u8; 0x7F],

    /// GPU
    pub gpu: Gpu,

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
}

impl Bus {
    pub fn new(rom: &[u8]) -> Bus {
        // TODO: Initialize the WRAM with the size specified by the cartridge
        let wram = iter::repeat(0).take(0x2000).collect();

        let mut rom_buffer = box [0; 0x8000];
        rom_buffer[..rom.len()].copy_from_slice(rom);

        let mut bus = Bus {
            rom_buffer,
            wram,
            ..Default::default()
        };

        // Startup sequence

        bus.mem_write(0xFF05, 0x00); // TIMA
        bus.mem_write(0xFF06, 0x00); // TMA
        bus.mem_write(0xFF07, 0x00); // TAC
        bus.mem_write(0xFF10, 0x80); // NR10
        bus.mem_write(0xFF11, 0xBF); // NR11
        bus.mem_write(0xFF12, 0xF3); // NR12
        bus.mem_write(0xFF14, 0xBF); // NR14
        bus.mem_write(0xFF16, 0x3F); // NR21
        bus.mem_write(0xFF17, 0x00); // NR22
        bus.mem_write(0xFF19, 0xBF); // NR24
        bus.mem_write(0xFF1A, 0x7F); // NR30
        bus.mem_write(0xFF1B, 0xFF); // NR31
        bus.mem_write(0xFF1C, 0x9F); // NR32
        bus.mem_write(0xFF1E, 0xBF); // NR33
        bus.mem_write(0xFF20, 0xFF); // NR41
        bus.mem_write(0xFF21, 0x00); // NR42
        bus.mem_write(0xFF22, 0x00); // NR43
        bus.mem_write(0xFF23, 0xBF); // NR44
        bus.mem_write(0xFF24, 0x77); // NR50
        bus.mem_write(0xFF25, 0xF3); // NR51
        bus.mem_write(0xFF26, 0xF1); // NR52
        bus.mem_write(0xFF40, 0x91); // LCDC
        bus.mem_write(0xFF42, 0x00); // SCY
        bus.mem_write(0xFF43, 0x00); // SCX
        bus.mem_write(0xFF45, 0x00); // LYC
        bus.mem_write(0xFF47, 0xFC); // BGP
        bus.mem_write(0xFF48, 0xFF); // OBP0
        bus.mem_write(0xFF49, 0xFF); // OBP1
        bus.mem_write(0xFF4A, 0x00); // WY
        bus.mem_write(0xFF4B, 0x00); // WX
        bus.mem_write(0xFFFF, 0x00); // IE

        bus
    }

    /// Ticks the IO devices
    pub fn tick(&mut self, clocks: u8) -> u8 {
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
            0x0000..=0x7FFF => self.rom_buffer[addr as usize],

            0x8000..=0x9FFF => self.gpu.mem_read(addr),

            0xA000..=0xBFFF => self.sram[(addr & 0x5FFF) as usize],

            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram[(addr & 0x0FFF) as usize],
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wram[(addr & 0x0FFF | 0x1000) as usize],

            0xFE00..=0xFE9F => self.gpu.mem_read(addr),

            0xFEA0..=0xFEFF => 0, // unused

            0xFF0F => self.iflag.bits(),

            0xFF01..=0xFF02 => self.serial.mem_read(addr),
            0xFF04..=0xFF07 => self.timer.mem_read(addr),

            0xFF46 => 0,
            0xFF40..=0xFF4B => self.gpu.mem_read(addr),

            0xFF00..=0xFF7F => self.io_registers[(addr & 0xFF) as usize],

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize],

            0xFFFF => self.ienable.bits(),
        }
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.rom_buffer[addr as usize] = value,

            0x8000..=0x9FFF => self.gpu.mem_write(addr, value),

            0xA000..=0xBFFF => self.sram[(addr & 0x5FFF) as usize] = value,

            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram[(addr & 0x0FFF) as usize] = value,
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.wram[(addr & 0x0FFF | 0x1000) as usize] = value
            }

            0xFE00..=0xFE9F => self.gpu.mem_write(addr, value),

            0xFEA0..=0xFEFF => (), // unused

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

            0xFF00..=0xFF7F => self.io_registers[(addr & 0xFF) as usize] = value,

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize] = value,

            0xFFFF => self.ienable = InterruptFlags::from_bits_truncate(value),
        }
    }
}

impl Default for Bus {
    fn default() -> Bus {
        Bus {
            rom_buffer: box [0; 0x8000],
            sram: box [0; 0x2000],
            vram: box [0; 0x2000],
            oam: [0; 0xA0],

            wram: Vec::new(),
            zram: [0; 0x7F],

            io_registers: [0; 0x80],
            timer: Timer::default(),
            serial: Serial::default(),
            gpu: Gpu::default(),

            iflag: Default::default(),
            ienable: Default::default(),
        }
    }
}

use std::iter;

use crate::{
    gb_mode::GbMode,
    io::{Joypad, Serial, Timer},
    ppu::Gpu,
};

use super::{
    cartridge::Cartridge,
    dma::{Dma, DmaMode},
    Mbc, MemoryAccess,
};

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
    pub cartridge: Cartridge,

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
    // io_registers: [u8; 0x80],

    /// Interrupt Flag (IF) \
    /// 0xFF0F
    pub iflag: u8,
    /// Interrupt Enable (IE) \
    /// 0xFFFF
    pub ienable: u8,

    // CGB registers
    // Direct Memory Access registers
    dma: Dma,
    /// WRAM Bank
    wram_bank: usize,

    speed: u8,
    speed_switch: bool,
}

impl Bus {
    pub fn new(rom: &[u8]) -> Bus {
        let cartridge = Cartridge::new(rom);

        Self::from_cartridge(cartridge)
    }

    pub fn from_cartridge(cartridge: Cartridge) -> Bus {
        let gb_mode = cartridge.gb_mode;

        let wram = iter::repeat(0).take(WRAM_SIZE).collect();
        let zram = [0; ZRAM_SIZE];

        let mut bus = Bus {
            gb_mode,

            wram,
            zram,

            cartridge,
            gpu: Gpu::new(gb_mode),
            joypad: Default::default(),
            serial: Default::default(),
            timer: Default::default(),
            // io_registers: [0; 0x80],
            ienable: Default::default(),
            iflag: Default::default(),

            wram_bank: 1,
            dma: Default::default(),
            speed: 1,
            speed_switch: false,
        };
        log::info!("Loaded cartridge: {:?}", bus.cartridge);
        log::info!("Game Boy mode: {gb_mode:?}");

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

    /// Sync the IO devices
    pub fn sync(&mut self, clocks: u32) -> u32 {
        let dma_clocks = self.start_dma();

        let Bus {
            ref speed,
            iflag,

            timer,
            joypad,
            gpu,
            serial,
            ..
        } = self;

        let speed = *speed;

        let cpu_clocks = clocks + dma_clocks * (speed as u32);
        let gpu_clocks = clocks / (speed as u32) + dma_clocks;

        // update the timer
        timer.sync(cpu_clocks);
        *iflag |= timer.interrupt;
        timer.interrupt = 0;

        *iflag |= joypad.interrupt;
        joypad.interrupt = 0;

        // update the gpu
        gpu.sync(gpu_clocks);
        *iflag |= gpu.interrupt;
        gpu.interrupt = 0;

        *iflag |= serial.interrupt;
        serial.interrupt = 0;

        gpu_clocks
    }

    pub fn switch_speed(&mut self) {
        if self.speed_switch {
            self.speed = [2, 1][(self.speed - 1) as usize];
        }
        self.speed_switch = false;
    }

    fn start_dma(&mut self) -> u32 {
        match self.dma.dma_mode {
            Some(DmaMode::Gdma) => self.start_gdma(),
            Some(DmaMode::Hdma) => self.start_hdma(),
            None => 0x00,
        }
    }
    fn start_hdma(&mut self) -> u32 {
        if !self.gpu.hblanking {
            return 0;
        }

        self.dma_cpblk(0x10);

        self.dma.dma_length -= 1;

        if self.dma.dma_length == 0x7F {
            self.dma.dma_mode = None
        }

        0x08
    }
    fn start_gdma(&mut self) -> u32 {
        let gdma_len = 1 + self.dma.dma_length as u16;

        let blk_size = gdma_len * 0x10;
        self.dma_cpblk(blk_size);

        self.dma.dma_length = 0x7F;
        self.dma.dma_mode = None;

        0x08 * gdma_len as u32
    }
    fn dma_cpblk(&mut self, blk_size: u16) {
        let src_addr = self.dma.src;

        for i in 0x00..blk_size {
            let src = self.mem_read(src_addr + i);
            self.mem_write(self.dma.dst + i, src)
        }
        self.dma.src += blk_size;
        self.dma.dst += blk_size;
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
                self.wram[(self.wram_bank * 0x1000) | (addr & 0x0FFF) as usize]
            }

            0xFE00..=0xFE9F => self.gpu.mem_read(addr),

            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => self.serial.mem_read(addr),
            0xFF04..=0xFF07 => self.timer.mem_read(addr),

            0xFF0F => self.iflag,

            0xff4d => ((self.speed & 0x02) << 6) | self.speed_switch as u8,
            0xff40..=0xff4f => self.gpu.mem_read(addr),
            0xff51..=0xff55 => self.dma.mem_read(addr),
            0xff68..=0xff6b => self.gpu.mem_read(addr),

            0xFF70 => self.wram_bank as u8,

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize],

            0xFFFF => self.ienable,
            _ => 0,
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

            0xFF00 => self.joypad.write(value),

            0xFF0F => self.iflag = value,

            0xFF01..=0xFF02 => self.serial.mem_write(addr, value),
            0xFF04..=0xFF07 => self.timer.mem_write(addr, value),

            0xFF46 => {
                let src_addr = (value as u16) << 8;
                for i in 0x00..=0x9F {
                    let dst_addr = 0xFE00 + i;
                    let src = self.mem_read(src_addr + i);

                    self.mem_write(dst_addr, src);
                }
            }
            0xff40..=0xff4b | 0xff4f => self.gpu.mem_write(addr, value),
            0xff4d => {
                if value & 1 != 0 {
                    self.speed_switch = true
                }
            }
            0xff51..=0xff55 => self.dma.mem_write(addr, value),
            0xff68..=0xff6b => self.gpu.mem_write(addr, value),

            0xFF70 => {
                self.wram_bank = if (value & 0x7) == 0 {
                    1
                } else {
                    (value & 0x7) as usize
                }
            }

            0xFF80..=0xFFFE => self.zram[(addr & 0x7F) as usize] = value,

            0xFFFF => self.ienable = value,
            _ => {}
        }
    }
}

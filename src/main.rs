use std::fs;

use sixels_gb::{bus::Bus, cpu::Cpu};

/*
 * BASIC INFORMATION
 *
 * RAM size: 8KiB
 * VRAM size: 8KiB
 * Resolution: 160x144 (20x18 tiles)
 * Clock freq: 4.194304 MHz
 *
 */

fn main() {
    // Get the ROM path from the first argument
    let mut args = std::env::args().skip(1);
    let rom_path = args
        .next()
        .expect("You should pass the rom path as the first argument.");

    // Load the ROM file into the memory
    let rom_buffer = fs::read(rom_path).expect("Could not read the rom path.");
    let bus = Bus::new(&rom_buffer);

    Cpu::new(bus).run();
}

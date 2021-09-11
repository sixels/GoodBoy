use std::fs;

use sixels_gb::cpu::Cpu;

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
    let rom_path = args.next().expect("You should pass the rom path as the first argument.");

    // Load the ROM and run
    let rom_buffer = fs::read(rom_path).expect("Could not read the rom path.");
    Cpu::new(&rom_buffer)
        .run();
}

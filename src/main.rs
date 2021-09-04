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
    let buffer = fs::read("assets/roms/boot.gb").unwrap();

    let mut cpu = Cpu::with_bootstrap(&buffer);
    cpu.run();
}

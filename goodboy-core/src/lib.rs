#![feature(destructuring_assignment)]
#![feature(never_type)]
#![feature(box_syntax)]

pub mod cpu;
mod gb_mode;
pub mod io;
pub mod mmu;
pub mod ppu;
pub mod utils;
pub mod vm;

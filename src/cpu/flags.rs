use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct CpuFlags: u8 {
        // Zero Flag
        const Z = 1 << 7;
        // Add/Subtract Flag
        const N = 1 << 6;
        // Half Carry Flag
        const H = 1 << 5;
        // Carry Flag
        const C = 1 << 4;
    }
}

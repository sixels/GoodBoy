#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GbMode {
    DMG,
    CGB,
}

impl Default for GbMode {
    /// Gameboy Color is the default
    fn default() -> Self {
        Self::CGB
    }
}

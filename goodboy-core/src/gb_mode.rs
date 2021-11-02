#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GbMode {
    Dmg,
    Cgb,
}

impl Default for GbMode {
    /// Gameboy Color is the default
    fn default() -> Self {
        Self::Cgb
    }
}

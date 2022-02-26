#[derive(Debug, Clone, Copy)]
pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

pub struct Joypad {
    rows: (u8, u8),
    column: u8,
    data: u8,
    pub interrupt: u8,
}

impl Joypad {
    pub fn read(&self) -> u8 {
        match self.column {
            0x10 => self.rows.0,
            0x20 => self.rows.1,
            _ => 0,
        }
    }
    pub fn write(&mut self, value: u8) {
        self.column = value & 0x30;
    }

    #[rustfmt::skip]
    pub fn press_button(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right  => self.rows.1 &= 0xE,
            JoypadButton::Left   => self.rows.1 &= 0xD,
            JoypadButton::Up     => self.rows.1 &= 0xB,
            JoypadButton::Down   => self.rows.1 &= 0x7,
            JoypadButton::A      => self.rows.0 &= 0xE,
            JoypadButton::B      => self.rows.0 &= 0xD,
            JoypadButton::Select => self.rows.0 &= 0xB,
            JoypadButton::Start  => self.rows.0 &= 0x7,
        }
        self.update();
    }
    #[rustfmt::skip]
    pub fn release_button(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right  => self.rows.1 |= 0x1,
            JoypadButton::Left   => self.rows.1 |= 0x2,
            JoypadButton::Up     => self.rows.1 |= 0x4,
            JoypadButton::Down   => self.rows.1 |= 0x8,
            JoypadButton::A      => self.rows.0 |= 0x1,
            JoypadButton::B      => self.rows.0 |= 0x2,
            JoypadButton::Select => self.rows.0 |= 0x4,
            JoypadButton::Start  => self.rows.0 |= 0x8,
        }
        self.update()
    }

    fn update(&mut self) {
        let old_values = self.data & 0xF;
        let mut new_values = 0xF;

        if self.data & 0x10 == 0x00 {
            new_values &= self.rows.0;
        }
        if self.data & 0x20 == 0x00 {
            new_values &= self.rows.1;
        }

        if old_values == 0xF && new_values != 0xF {
            self.interrupt |= 0x10;
        }

        self.data = (self.data & 0xF0) | new_values;
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad {
            rows: (0x0F, 0x0F),
            column: 0,
            data: 0xFF,
            interrupt: 0,
        }
    }
}

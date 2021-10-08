bitflags::bitflags! {
    #[derive(Default)]
    pub struct LCDControl: u8 {
        /// 0 -> Off
        /// 1 -> On
        const LCD_POWER = 1 << 7;
        /// 0 -> 0x9800..=0x9BFF
        /// 1 -> 0x9C00..=0x9FFF
        const WIN_TILEMAP = 1 << 6;
        /// 0 -> Off
        /// 1 -> On
        const WIN_ENABLE = 1 << 5;
        /// 0 -> 0x8800..=0x9CFF
        /// 1 -> 0x8000..=0x8FFF
        const BG_WIN_TILESET = 1 << 4;
        /// 0 -> 0x9800..=0x9BFF
        /// 1 -> 0x9C00..=0x9FFF
        const BG_TILEMAP = 1 << 3;
        /// 0 -> 8x8
        /// 1 -> 8x16
        const SPRITE_SIZE = 1 << 2;
        /// 0 -> Off
        /// 1 -> On
        const SPRITES_ENABLE = 1 << 1;
        /// 0 -> Off
        /// 1 -> On
        const BG_ENABLE = 1 << 0;
    }
}

impl LCDControl {
    /// 0 -> 0x9800..=0x9BFF
    /// 1 -> 0x9C00..=0x9FFF
    #[inline(always)]
    pub fn lcd_on(&self) -> bool {
        self.contains(Self::LCD_POWER)
    }

    /// 0 -> 0x9800..=0x9BFF
    /// 1 -> 0x9C00..=0x9FFF
    #[inline(always)]
    pub fn win_tilemap(&self) -> u16 {
        if self.contains(Self::WIN_TILEMAP) {
            0x9C00
        } else {
            0x9800
        }
    }

    /// 0 -> Off
    /// 1 -> On
    #[inline(always)]
    pub fn win_on(&self) -> bool {
        self.contains(Self::WIN_ENABLE)
    }

    /// 0 -> 0x8800..=0x9CFF
    /// 1 -> 0x8000..=0x8FFF
    #[inline(always)]
    pub fn tileset_base(&self) -> u16 {
        if self.contains(Self::BG_WIN_TILESET) {
            0x8000
        } else {
            0x8800
        }
    }

    /// 0 -> 0x9800..=0x9BFF
    /// 1 -> 0x9C00..=0x9FFF
    #[inline(always)]
    pub fn bg_tilemap(&self) -> u16 {
        if !self.contains(Self::BG_TILEMAP) {
            0x9800
        } else {
            0x9C00
        }
    }

    /// 0 -> 8x8
    /// 1 -> 8x16
    #[inline(always)]
    pub fn sprite_size(&self) -> u32 {
        if self.contains(Self::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }

    /// 0 -> Off
    /// 1 -> On
    #[inline(always)]
    pub fn sprites_on(&self) -> bool {
        self.contains(Self::SPRITES_ENABLE)
    }

    /// 0 -> Off
    /// 1 -> On
    #[inline(always)]
    pub fn bg_enabled(&self) -> bool {
        self.contains(Self::BG_ENABLE)
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct LCDStatus: u8 {
        const SCAN_LINE_CHECK = 1 << 6;
        const OAM_CHECK = 1 << 5;
        const VBLANK_CHECK = 1 << 4;
        const HBLANK_CHECK = 1 << 3;
    }
}

impl LCDStatus {
    pub fn scanline_check(&self) -> bool {
        self.contains(Self::SCAN_LINE_CHECK)
    }
    pub fn oam_check(&self) -> bool {
        self.contains(Self::OAM_CHECK)
    }
    pub fn vblank_check(&self) -> bool {
        self.contains(Self::VBLANK_CHECK)
    }
    pub fn hblank_check(&self) -> bool {
        self.contains(Self::HBLANK_CHECK)
    }
}

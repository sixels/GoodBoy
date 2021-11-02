pub enum UnsignedValue {
    U8(u8),
    U16(u16),
}

impl UnsignedValue {
    pub fn is_u8(&self) -> bool {
        matches!(self, Self::U8(_))
    }

    pub fn is_u16(&self) -> bool {
        matches!(self, Self::U16(_))
    }

    pub fn unwrap_u8(&self) -> u8 {
        match self {
            Self::U8(v) => *v,
            _ => panic!("Unwrap failed"),
        }
    }
    pub fn unwrap_u16(&self) -> u16 {
        match self {
            Self::U16(v) => *v,
            _ => panic!("Unwrap failed"),
        }
    }
}

impl From<u8> for UnsignedValue {
    fn from(v: u8) -> Self {
        Self::U8(v)
    }
}

impl From<u16> for UnsignedValue {
    fn from(v: u16) -> Self {
        Self::U16(v)
    }
}

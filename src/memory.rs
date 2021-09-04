pub trait MemoryAccess {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_read_word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
    }

    fn mem_write(&mut self, addr: u16, value: u8);
    fn mem_write_word(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.mem_write(addr, bytes[0]);
        self.mem_write(addr + 1, bytes[1]);
    }
}

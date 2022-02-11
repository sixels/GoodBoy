// use std::io::Write;

use crate::mmu::MemoryAccess;

#[derive(Default)]
pub struct Serial {
    data: u8,
    control: u8,
    pub interrupt: u8,
}

impl Serial {
    fn display(&self) {
        let data = self.data;
        let char_data = char::from_u32(data as u32).unwrap().escape_default();

        log::debug!("Serial Ouput: 0x{data:02X} ({char_data})",);

        // write to serial message to stderr
        // let mut stderr = std::io::stderr();
        // stderr.write_all(&[data]).unwrap();
        // std::io::stderr().flush().unwrap();
    }
}

impl MemoryAccess for Serial {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => panic!("Invalid Serial address"),
        }
    }
    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF01 => self.data = value,
            0xFF02 => {
                if value == 0x81 {
                    self.display();

                    self.data = value;
                    self.interrupt = 0x08;
                }
            }
            _ => panic!("Invalid Serial address"),
        }
    }
}

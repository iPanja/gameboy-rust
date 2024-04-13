use super::Memory;

pub struct Bus {
    pub ram: Memory,
}

impl Bus {
    pub fn new() -> Self {
        Bus { ram: Memory::new() }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        self.ram.read_byte(address)
    }

    pub fn ram_write_byte(&mut self, address: u16, byte: u8) {
        self.ram.write_byte(address, byte);
    }

    pub fn ram_read_word(&self, address: u16) -> u16 {
        self.ram.read_word(address)
    }

    pub fn ram_write_word(&mut self, address: u16, word: u16) {
        self.ram.write_word(address, word);
    }

    pub fn ram_load_rom(&mut self, buffer: &Vec<u8>, addr: usize) {
        for i in 0..buffer.len() {
            self.ram_write_byte((Memory::START_ADDR + i + addr) as u16, buffer[i]);
        }
    }
}

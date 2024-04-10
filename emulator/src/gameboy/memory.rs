const RAM_SIZE: usize = 65536;

pub struct Memory {
    ram: [u8; RAM_SIZE], // 0x0000 to 0xFFFF
}

impl Memory {
    pub const START_ADDR: usize = 0x0000;

    pub fn new() -> Self {
        Memory { ram: [0; RAM_SIZE] }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.ram[address as usize] = byte;
    }
}

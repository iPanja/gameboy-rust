const RAM_SIZE: usize = 65536;
//const RAM_SIZE: usize =

use serde_big_array::BigArray;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Memory {
    #[serde(with = "BigArray")]
    pub ram: [u8; RAM_SIZE], // 0x0000 to 0xFFFF
}

impl Memory {
    pub const START_ADDR: usize = 0x0000;

    pub fn new() -> Self {
        Memory { ram: [0; RAM_SIZE] }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        //println!("r: {address}");
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        //println!("w: {address}");
        self.ram[address as usize] = byte;
    }

    pub fn read_word(&self, address: u16) -> u16 {
        ((self.read_byte(address + 1) as u16) << 8) | (self.read_byte(address) as u16)
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0xFF) as u8);
        self.write_byte(address + 1, (word >> 8) as u8);
    }
}

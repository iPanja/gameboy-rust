use serde_big_array::BigArray;

use super::MBC;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MBC0 {
    #[serde(with = "BigArray")]
    rom: [u8; 0x7FFF + 1],
}

impl MBC0 {
    pub fn new() -> Self {
        MBC0 {
            rom: [0; 0x7FFF + 1],
        }
    }
}

#[typetag::serde]
impl MBC for MBC0 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0xA000..=0xBFFF => 0, //self.ram[(addr - 0xA000) as usize],
            _ => panic!("Unsupported MBC0 memory read @{:#X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000..=0x7FFF => (), //panic!("writing to addr: {:#X}", addr), //self.rom[addr as usize] = byte,
            0xA000..=0xBFFF => (), //self.ram[(addr - 0xA000) as usize] = byte,
            _ => panic!("Unsupported MBC0 memory access (write) @{:#X}", addr),
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        for i in 0..rom_data.len() {
            self.rom[i] = rom_data[i];
        }
    }
}

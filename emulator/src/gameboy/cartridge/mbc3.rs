use serde_big_array::BigArray;

use super::MBC;

/// Up to 2MB ROM (128 banks)
/// Up to 32KB RAM (4 banks)
/// Built-in Real Time Clock (RTC)

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MBC3 {
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
            0x000..=0x3FFF => self.rom[addr as usize], // ROM Bank 00 (Read Only)
            0x4000..=0x7FFF => self.rom[addr as usize], // ROM Bank 01-7F (Read Only)
            0xA000..=0xBFFF => 0,                      // RAM Bank 00-03, if any (Read/Write)
            _ => panic!("Unsupported MBC3 memory read @{:#X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000..=0x7FFF => (),
            0xA000..=0xBFFF => (),
            _ => panic!("Unsupported MBC3 memory access (write) @{:#X}", addr),
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        for i in 0..rom_data.len() {
            self.rom[i] = rom_data[i];
        }
    }
}

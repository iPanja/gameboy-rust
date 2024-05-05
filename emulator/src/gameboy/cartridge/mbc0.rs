use super::MBC;

pub struct MBC0 {
    rom: [u8; 0x7FFF + 1],
    //ram: [u8; 0xBFFF - 0xA000],

    //is_ram_enabled: bool,
}

impl MBC0 {
    pub fn new() -> Self {
        MBC0 {
            rom: [0; 0x7FFF + 1],
            //ram: [0; 0xBFFF - 0xA000],
            //is_ram_enabled: false,
        }
    }
}

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
            0x0000..=0x7FFF => self.rom[addr as usize] = byte,
            0xA000..=0xBFFF => (), //self.ram[(addr - 0xA000) as usize] = byte,
            _ => panic!("Unsupported MBC0 memory access (write) @{:#X}", addr),
        }
    }
}

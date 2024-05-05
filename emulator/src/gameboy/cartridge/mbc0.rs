use super::MBC;

pub struct MBC0 {
    rom: [u8; 0x7FFF + 1],
    ram: [u8; 0xBFFF - 0xA000],

    is_ram_enabled: bool,
}

impl MBC0 {
    pub fn new() -> Self {
        MBC0 {
            rom: [0; 0x7FFF + 1],
            ram: [0; 0xBFFF - 0xA000],
            is_ram_enabled: false,
        }
    }
}

impl MBC for MBC0 {
    fn read_byte(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        //println!("throwing away write... @{:#X}, value: {:#X}", addr, byte);
        self.rom[addr as usize] = byte;
    }
}

use super::MBC;

pub struct MBC1 {
    rom: [u8; 512000],
    ram: [u8; 32000],
    is_ram_enabled: bool,
    rom_bank_index: u8,
    rom_bank_count: u8,
}

impl MBC1 {
    /*pub*/
    fn new() -> Self {
        MBC1 {
            rom: [0; 512000],
            ram: [0; 32000],
            is_ram_enabled: false,
            rom_bank_index: 1,
            rom_bank_count: 1,
        }
    }
}

impl MBC for MBC1 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0xA000..=0xBFFF => {
                if self.is_ram_enabled {
                    self.ram[(addr - 0xA000) as usize]
                } else {
                    0xFF
                }
            }
            _ => panic!("Unsupported MBC0 memory read @{:#X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000..=0x3FFF => {
                // RAM Enable (Write Only)
                match addr {
                    0x0000..=0x1FFF => {
                        if byte & 0xF == 0xA {
                            self.is_ram_enabled = true;
                        } else {
                            self.is_ram_enabled = false;
                        }
                    }
                    // 0x2000..=0x3FFF
                    _ => {
                        self.rom_bank_index = byte & 0b0001;
                        if self.rom_bank_index == 0 {
                            self.rom_bank_index = 1;
                        } else if self.rom_bank_index > self.rom_bank_count {
                            // MASK
                        }
                    }
                }
            } // Read-only ROM Bank X0
            0x4000..=0x7FFF => (), // Read-only ROM Bank 01-7F
            0xA000..=0xBFFF => {
                // RAM Bank 00-03 (if any)
                if self.is_ram_enabled {
                    self.ram[(addr - 0xA000) as usize] = byte;
                } else {
                    ()
                }
            }
            _ => panic!("Unsupported MBC0 memory access (write) @{:#X}", addr),
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        for i in 0..rom_data.len() {
            self.rom[i] = rom_data[i];
        }
    }
}

use super::{Memory, PPU};

pub struct Bus {
    pub ram: Memory,
    pub ppu: PPU,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Memory::new(),
            ppu: PPU::new(),
        }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x97FF => self.ppu.read_byte((address - 0x8000) as usize), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.read_byte((address - 0x6000) as usize), // PPU - OAM
            _ => self.ram.read_byte(address),
        }
    }

    pub fn ram_write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0x97FF => self.ppu.write_byte((address - 0x8000) as usize, byte), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.write_byte((address - 0x6000) as usize, byte), // PPU - OAM
            /*0xFF01..=0xFF02 => {
                println!("{:?}", byte as char);
            }*/
            _ => self.ram.write_byte(address, byte),
        }
    }

    pub fn ram_read_word(&self, address: u16) -> u16 {
        match address {
            0x8000..=0x97FF => self.ppu.read_byte((address - 0x8000) as usize) as u16, // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.read_byte((address - 0x6000) as usize) as u16, // PPU - OAM
            _ => self.ram.read_word(address),
        }
    }

    pub fn ram_write_word(&mut self, address: u16, word: u16) {
        match address {
            0x8000..=0x97FF => self.ppu.write_byte((address - 0x8000) as usize, word as u8), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.write_byte((address - 0x6000) as usize, word as u8), // PPU - OAM
            _ => self.ram.write_word(address, word),
        }
    }

    pub fn ram_load_rom(&mut self, buffer: &Vec<u8>, addr: usize) {
        for i in 0..buffer.len() {
            self.ram_write_byte((Memory::START_ADDR + i + addr) as u16, buffer[i]);
        }
    }
}

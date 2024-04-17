use super::{Memory, PPU};

pub struct Bus {
    pub ram: Memory,
    pub ppu: PPU,
    dbg: Vec<char>
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Memory::new(),
            ppu: PPU::new(),
            dbg: Vec::new()
        }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        if 0xE000 <= address && address <= 0xFDFF { println!("reading echo ram"); }

        match address {
            0x8000..=0x97FF => self.ppu.read_byte((address - 0x8000) as usize), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.read_byte((address - 0x6000) as usize), // PPU - OAM
            _ => self.ram.read_byte(address),
        }
    }

    pub fn ram_write_byte(&mut self, address: u16, byte: u8) {
        if 0xE000 <= address && address <= 0xFDFF { println!("writing echo ram"); }

        match address {
            0x8000..=0x97FF => self.ppu.write_byte((address - 0x8000) as usize, byte), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self.ppu.write_byte((address - 0x6000) as usize, byte), // PPU - OAM
            _ => self.ram.write_byte(address, byte),
        }
    }

    pub fn ram_read_word(&self, address: u16) -> u16 {
        ((self.ram_read_byte(address + 1) as u16) << 8) | (self.ram_read_byte(address) as u16)
    }

    pub fn ram_write_word(&mut self, address: u16, word: u16) {
        self.ram_write_byte(address, (word & 0xFF) as u8);
        self.ram_write_byte(address + 1, (word >> 8) as u8);
    }

    pub fn ram_load_rom(&mut self, buffer: &Vec<u8>, addr: usize) {
        for i in 0..buffer.len() {
            self.ram_write_byte((Memory::START_ADDR + i + addr) as u16, buffer[i]);
        }
    }

    pub fn debug(&mut self){
        // Build
        if self.ram_read_byte(0xFF02) == 0x81 {
            let c_byte = self.ram_read_byte(0xFF01);
            let c = char::from(c_byte);
            self.dbg.push(c);
            self.ram_write_byte(0xFF02, 0);
        }

        // Print
        let result: String = self.dbg.iter().collect();
        println!("Serial Port: {}", result);
    }
}

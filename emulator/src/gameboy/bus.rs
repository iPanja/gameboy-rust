use super::{
    cartridge::{MBC, MBC0},
    Interrupt, Memory, Timer, PPU,
};

const BOOT_ROM_SIZE: u16 = 0x100;

pub struct Bus {
    ram: Memory,
    pub ppu: PPU,
    pub timer: Timer,
    pub dbg: Vec<char>,
    pub mbc: Box<dyn MBC>,
    is_boot_rom_mapped: bool,
    boot_rom: [u8; BOOT_ROM_SIZE as usize],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Memory::new(),
            ppu: PPU::new(),
            timer: Timer::new(),
            dbg: Vec::new(),
            mbc: Box::new(MBC0::new()),
            is_boot_rom_mapped: false,
            boot_rom: [0; BOOT_ROM_SIZE as usize],
        }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        if 0xE000 <= address && address <= 0xFDFF {
            //println!("reading echo ram");
        }

        match address {
            0x0000..=0x7FFF => {
                if self.is_boot_rom_mapped && address < BOOT_ROM_SIZE {
                    self.boot_rom[address as usize]
                } else {
                    self.mbc.read_byte(address)
                }
            }
            0xA000..=0xBFFF => self.mbc.read_byte(address),

            0x8000..=0x9FFF => self // 0x97FF
                .ppu
                .read_byte((address - 0x8000) as usize, address as usize), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => self
                .ppu
                .read_byte((address - 0x5200) as usize, address as usize), // PPU - OAM
            0xFF40..=0xFF4B => self
                .ppu
                .read_byte((address - 0xFF40) as usize, address as usize), // PPU - Internal Registers
            0xFF04..=0xFF07 => self.timer.read_byte((address - 0xFF04) as usize), // Timer and Divider Registers
            _ => self.ram.read_byte(address),
        }
    }

    pub fn ram_write_byte(&mut self, address: u16, byte: u8) {
        if 0xE000 <= address && address <= 0xFDFF {
            println!("writing echo ram");
        }

        match address {
            0x0000..=0x7FFF => self.mbc.write_byte(address, byte),
            0xA000..=0xBFFF => self.mbc.write_byte(address, byte),
            0xFF50 => self.is_boot_rom_mapped = false,

            0x8000..=0x9FFF => {
                // 0x97FF
                self.ppu
                    .write_byte((address - 0x8000) as usize, address as usize, byte)
            } // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xFE00..=0xFE9F => {
                self.ppu
                    .write_byte((address - 0x5200) as usize, address as usize, byte)
            } // PPU - OAM
            0xFF40..=0xFF45 => {
                self.ppu
                    .write_byte((address - 0xFF40) as usize, address as usize, byte)
            }
            0xFF46 => self.dma_transfer(address),
            0xFF47..=0xFF4B => {
                self.ppu
                    .write_byte((address - 0xFF40) as usize, address as usize, byte)
            }
            /*0xFF40..=0xFF4B => {
                self.ppu
                    .write_byte((address - 0xFF40) as usize, address as usize, byte)
            }*/ // PPU - Internal Registers
            0xFF04..=0xFF07 => self.timer.write_byte((address - 0xFF04) as usize, byte), // Timer and Divider Registers
            _ => self.ram.write_byte(address, byte),
        }
    }

    fn dma_transfer(&mut self, address: u16) {
        let real_addr = address << 8;
        for index in 0..0xA0 {
            let value = self.ram_read_byte(real_addr + index);
            self.ram_write_byte(0xFE00 + index, value);
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
            //self.ram_write_byte((Memory::START_ADDR + i + addr) as u16, buffer[i]);
            self.mbc.write_byte((addr + i) as u16, buffer[i]);
        }
    }

    pub fn ram_load_boot_rom(&mut self, buffer: &Vec<u8>) {
        for i in 0..buffer.len() {
            self.boot_rom[i] = buffer[i];
        }
        self.is_boot_rom_mapped = true;
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        let mask = interrupt.get_flag_mask();
        let ifr = self.ram_read_byte(0xFF0F); // IF_REG

        self.ram_write_byte(0xFF0F, ifr | mask);
    }

    pub fn tick(&mut self, cycles: u8) {
        self.timer.tick(cycles);

        if let Some(interrupt) = self.ppu.tick(cycles as u16) {
            self.trigger_interrupt(interrupt);
        }
    }

    pub fn debug(&mut self) {
        // Build
        if self.ram_read_byte(0xFF02) == 0x81 {
            let c_byte = self.ram_read_byte(0xFF01);
            let c = char::from(c_byte);
            self.dbg.push(c);
            self.ram_write_byte(0xFF02, 0);

            // Print
            /*
            let result: String = self.dbg.iter().collect();
            println!("Serial Port: {}", result);

            if result.contains("Failed") {
                //panic!("Test failed!");
            } else if result.contains("Passed") {
                //std::process::exit(0);
            }
            */
        }
    }
}

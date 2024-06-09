use super::{
    cartridge::{MBC, MBC0},
    interrupt, Interrupt, Joypad, Memory, Timer, PPU,
};
use serde_big_array::BigArray;

const BOOT_ROM_SIZE: u16 = 0x100;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Bus {
    ram: Memory,
    pub ppu: PPU,
    pub timer: Timer,
    pub dbg: Vec<char>,
    pub mbc: Box<dyn MBC>,
    pub joypad: Joypad,
    is_boot_rom_mapped: bool,
    #[serde(with = "BigArray")]
    boot_rom: [u8; BOOT_ROM_SIZE as usize],
    dma_address_upper: u8,
    #[serde(with = "BigArray")]
    wram: [u8; 0x8000],
    #[serde(with = "BigArray")]
    hram: [u8; 0x7F],
    serial: [u8; 2],
    interrupt_flags: u8,
    interrupts_enabled: u8,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Memory::new(),
            ppu: PPU::new(),
            timer: Timer::new(),
            dbg: Vec::new(),
            mbc: Box::new(MBC0::new()),
            joypad: Joypad::new(),
            is_boot_rom_mapped: false,
            boot_rom: [0; BOOT_ROM_SIZE as usize],
            dma_address_upper: 0,
            wram: [0; 0x8000],
            hram: [0; 0x7F],
            serial: [0xFF, 0], //[0; 2],
            interrupt_flags: 0,
            interrupts_enabled: 0,
        }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => {
                if self.is_boot_rom_mapped && address < BOOT_ROM_SIZE {
                    self.boot_rom[address as usize]
                } else {
                    self.mbc.read_byte(address)
                }
            }
            0x8000..=0x9FFF => self // 0x97FF
                .ppu
                .read_byte((address - 0x8000) as usize, address as usize), // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xA000..=0xBFFF => self.mbc.read_byte(address),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize], // Work RAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize], // Echo RAM
            0xFE00..=0xFE9F => self
                .ppu
                .read_byte((address - 0x5200) as usize, address as usize), // PPU - OAM
            0xFEA0..=0xFEFF => 0x00,                                   // TODO: Not Usable
            0xFF00..=0xFF7F => {
                // IO Registers
                match address {
                    0xFF00 => self.joypad.read_byte(), // Joypad Input
                    0xFF01..=0xFF02 => self.serial[(address & 0x1) as usize], // SERIAL
                    0xFF04..=0xFF07 => self.timer.read_byte((address - 0xFF04) as usize), // Timer
                    0xFF0F => self.interrupt_flags,
                    0xFF10..=0xFF3F => 0x0, // TODO: Audio & Audio Wave
                    0xFF40..=0xFF45 => self
                        .ppu
                        .read_byte((address - 0xFF40) as usize, address as usize), // PPU
                    0xFF46 => self.dma_address_upper,
                    0xFF47..=0xFF4B => self
                        .ppu
                        .read_byte((address - 0xFF40) as usize, address as usize), // PPU
                    0xFF4F => 0xFF, // TODO: VRAM Bank Select - CBG Only, Need Not Implement?
                    0xFF50 => self.is_boot_rom_mapped as u8,
                    _ => self.ram.read_byte(address), // 0x0
                }
            }
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts_enabled, // TODO: Access CPU for value?
            _ => self.ram.read_byte(address),  // 0x0
        }
    }

    pub fn ram_write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x7FFF => self.mbc.write_byte(address, byte),

            0x8000..=0x9FFF => {
                // 0x97FF
                self.ppu
                    .write_byte((address - 0x8000) as usize, address as usize, byte)
            } // PPU - Tile RAM & Background Map (Division at 0x9800)
            0xA000..=0xBFFF => self.mbc.write_byte(address, byte),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = byte, // Work RAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = byte, // Echo RAM
            0xFE00..=0xFE9F => {
                self.ppu
                    .write_byte((address - 0x5200) as usize, address as usize, byte)
            } // PPU - OAM
            0xFEA0..=0xFEFF => (),                                            // TODO: Not Usable
            0xFF00..=0xFF7F => {
                // IO Registers
                match address {
                    0xFF00 => self.joypad.write_byte(byte), // Joypad Input
                    0xFF01..=0xFF02 => self.serial[(address & 0x1) as usize] = byte, // SERIAL
                    0xFF04..=0xFF07 => self.timer.write_byte((address - 0xFF04) as usize, byte), // Timer and Divider Registers
                    0xFF0F => self.interrupt_flags = byte,
                    0xFF10..=0xFF3F => (), // TODO: Audio
                    0xFF40..=0xFF45 => {
                        self.ppu
                            .write_byte((address - 0xFF40) as usize, address as usize, byte)
                    }
                    0xFF46 => self.dma_transfer(byte),
                    0xFF47..=0xFF4B => {
                        self.ppu
                            .write_byte((address - 0xFF40) as usize, address as usize, byte)
                    }
                    0xFF4F => (), // TODO: VRAM Bank Select - CBG Only, Need Not Implement?
                    0xFF50 => self.is_boot_rom_mapped = false,
                    _ => self.ram.write_byte(address, byte), // ()
                }
            }
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = byte,
            0xFFFF => self.interrupts_enabled = byte,
            _ => self.ram.write_byte(address, byte), // ()
        }
    }

    fn dma_transfer(&mut self, address: u8) {
        self.dma_address_upper = address;

        let real_addr = (address as u16) << 8;
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
        self.mbc.load_rom(buffer);
    }

    pub fn ram_load_boot_rom(&mut self, buffer: &Vec<u8>) {
        for i in 0..buffer.len() {
            self.boot_rom[i] = buffer[i];
        }
        self.is_boot_rom_mapped = true;
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        //println!("Requesting interrupt: {:?}", interrupt);
        let interrupt_bit = interrupt.get_flag_mask();
        let ifr = self.ram_read_byte(0xFF0F); // IF_REG

        self.ram_write_byte(0xFF0F, ifr | interrupt_bit);
    }

    pub fn tick(&mut self, cycles: u8) {
        self.timer.tick(cycles);

        let ppu_interrupts = self.ppu.tick(cycles as u16);

        for interrupt in ppu_interrupts {
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
        }
    }
}

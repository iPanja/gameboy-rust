use super::{CartridgeHeader, MBC};
use serde_big_array::BigArray;

/// Max 16Mbit ROM (128 banks of 0x4000 bytes or 16KiB)
const ROM_BANK_SIZE: usize = 0x4000;

/// Max 256bit RAM (4 banks of 0x2000 bytes or 8KiB)
const RAM_BANK_SIZE: usize = 0x2000;

// More complicated implementation

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,

    /// **RAM gate register** - only lower nibble (3-0) is used, upper nibble is ignored during writes
    ///
    /// Used to enable access to the cartridge SRAM (if one exists).
    /// This access is *disabled* by default.
    ///
    /// Effectively, ram_access = ramg & 0x0F == 0xA, so this could instead be stored internally as a boolean
    ramg: u8,

    /// **Bank register 1** - 5 bit register (only bits 4-0 are used)
    ///
    /// Used as the lower 5 bits of the ROM bank number when reading from 0x4000-0x7FFF.
    /// Can not contain 0b0_0000, attempting to write 0 will instead write 1.
    /// So memory banks 0x00, 0x20, 0x40, 0x60 are impossible to read.
    bank1: u8,

    /// **Bank register 2** - 2 bit register (only bits 1-0 are used)
    ///
    /// Can be used as the upper bits of the ROM bank number, or as the 2-bit RAM bank number.
    bank2: u8,

    /// **Mode register**
    ///
    /// Determines how the BANK2 register value is used during memory accesses.
    ///
    /// 0b1 = BANK2 affects accesses to 0x0000-0x3FFF, 0x4000-0x7FFF, 0xA000-0xBFFF.
    ///
    /// 0b0 = BANK2 affects only accesses to 0x4000-0x7FFF.
    mode: bool,

    rom_bank_count: i32,
    ram_bank_count: i32,

    rom_mask: usize,
    ram_mask: usize,
}

impl MBC1 {
    /*pub*/
    pub fn new(ch: &CartridgeHeader) -> Self {
        println!("CH: {:?}", ch);
        MBC1 {
            rom: vec![0; ch.rom_size as usize * 1024], // rom_size * 320000?
            ram: vec![0; ch.ram_size as usize * 1024], // ram_size * 1024?

            ramg: 0,
            bank1: 1,
            bank2: 0,
            mode: false,

            rom_bank_count: ch.rom_bank_count,
            ram_bank_count: ch.ram_bank_count,

            rom_mask: MBC1::get_bit_mask(ch.rom_bank_count as usize),
            ram_mask: MBC1::get_bit_mask(ch.ram_bank_count as usize),
        }
    }
}

#[typetag::serde]
impl MBC for MBC1 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                // ROM Bank X0
                let bank_no: usize = if !self.mode {
                    // Mode 0b0
                    0
                } else {
                    // Mode 0b1
                    (self.bank2 << 5) as usize
                };
                self.rom_read_byte(bank_no, addr as usize - 0x000)
            }
            0x4000..=0x7FFF => {
                // Switchable ROM Bank
                // ROM Bank 01-7F
                let bank_no = (self.bank2 << 5) | self.bank1;
                self.rom_read_byte(bank_no as usize, addr as usize - 0x4000)
            }
            0xA000..=0xBFFF => {
                // RAM Bank 0-3
                if self.is_ram_accessible() {
                    let bank_no: usize = if !self.mode {
                        // Mode 0b0
                        0
                    } else {
                        // Mode 0b1
                        self.bank2 as usize
                    };
                    self.ram_read_byte(bank_no, (addr - 0xA000) as usize)
                } else {
                    0xFF
                }
            }
            _ => panic!("Unsupported MBC1 memory read @{:#X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // RAM Enable (Write Only)
                self.ramg = byte & 0xF; // Upper nibble is ignored during writes
            }
            0x2000..=0x3FFF => {
                // ROM Bank Index
                // Selects the lower 5 bits of ROM bank number
                // Writing 0x00 translates to 0x01
                self.bank1 = byte & 0b1_1111;
                if self.bank1 == 0 {
                    self.bank1 = 1;
                }
            }
            0x4000..=0x5FFF => {
                // RAM Bank Index
                // 2 bit register, depending on cartridge mode:
                //  > Selects RAM banks 0-3
                //  > Specifies the upper 2 bits of ROM bank
                self.bank2 = byte & 0b11;
            }
            0x6000..=0x7FFF => {
                // ROM/RAM Mode
                // Mode 0 - ROM Banking Mode (default)
                //  > Only RAM bank 0 can be accessed in this mode
                // Mode 1 - RAM Banking Mode
                //  > Only ROM banks 0x01-0x1F can be accessed in this mode
                //  > Other ROM banks will be changed to their corresponding in 0x01-0x1F by clearing the upper 2 bits
                self.mode = byte & 0x1 != 0;
            }
            0xA000..=0xBFFF => {
                // RAM Bank 00-03 (if any)
                if self.is_ram_accessible() {
                    self.ram_write_byte(self.bank2 as usize, (addr - 0xA000) as usize, byte);
                }
            }
            _ => panic!("Unsupported MBC0 memory access (write) @{:#X}", addr),
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        println!("loading {:?} bytes", rom_data.len());
        println!("\tROM SIZE: {:?}", self.rom.len());
        println!("\t\tMask: {:#b}", self.rom_mask);
        println!("\tRAM SIZE: {:?}", self.ram.len());
        println!("\t\tMask: {:#b}", self.ram_mask);

        for i in 0..rom_data.len() {
            self.rom[i] = rom_data[i];
        }
    }
}

impl MBC1 {
    /// Form bit mask for the current sized ROM
    ///
    /// *Ex:* 64KiB => 4 banks => 2 bits => mask: 0b11
    fn get_bit_mask(bank_count: usize) -> usize {
        let mask_size = f32::ceil((bank_count as f32).log2()) as usize;
        let mut mask = 0;

        for _ in 0..mask_size {
            mask <<= 1;
            mask |= 1;
        }

        mask
    }

    // Helper Methods
    fn is_ram_accessible(&self) -> bool {
        // We care only about the lower nibble
        // The upper nibble should always be 0 since I mask in write_byte, but I mask again just in case
        (self.ramg & 0x0F == 0b1010) && self.ram_bank_count != 0
    }

    // ROM R/W
    fn rom_read_byte(&self, bank_no: usize, offset: usize) -> u8 {
        let real_bank_no: usize = bank_no & self.rom_mask;
        self.rom[real_bank_no * ROM_BANK_SIZE + offset]
    }

    fn rom_write_byte(&mut self, bank_no: usize, offset: usize, byte: u8) {
        let real_bank_no: usize = bank_no & self.rom_mask;
        self.rom[real_bank_no * ROM_BANK_SIZE + offset] = byte;
    }

    // RAM R/W
    fn ram_read_byte(&self, bank_no: usize, offset: usize) -> u8 {
        let real_bank_no: usize = bank_no & self.ram_mask;
        self.ram[real_bank_no * RAM_BANK_SIZE + offset]
    }

    fn ram_write_byte(&mut self, bank_no: usize, offset: usize, byte: u8) {
        let real_bank_no: usize = bank_no & self.ram_mask;
        self.ram[real_bank_no * RAM_BANK_SIZE + offset] = byte;
    }
}

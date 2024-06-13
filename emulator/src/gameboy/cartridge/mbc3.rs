use super::{CartridgeHeader, MBC};
use serde_big_array::BigArray;

/// Max 16Mbit ROM (128 banks of 0x4000 bytes or 16KiB)
const ROM_BANK_SIZE: usize = 0x4000;

/// Max 256bit RAM (4 banks of 0x2000 bytes or 8KiB)
const RAM_BANK_SIZE: usize = 0x2000;

// More complicated implementation

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MBC3 {
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
    /// Used as the lower 7 bits of the ROM bank number when reading from 0x4000-0x7FFF.
    bank1: u8,

    /// **Bank register 2** - 2 bit register (only bits 1-0 are used)
    ///
    /// Accessing 0xA000-0xBFFF will depend on this value:
    /// 0x00-0x03: Accessing 2 bit register
    ///   > Can be used as the upper bits of the ROM bank number, or as the 2-bit RAM bank number.
    ///
    /// 0x08-0x0C: Accessing 0xA000-0xBFFF will read the RTC
    bank2: u8,

    /// **Mode register**
    ///
    /// Determines how the BANK2 register value is used during memory accesses.
    ///
    /// 0b1 = BANK2 affects accesses to 0x0000-0x3FFF, 0x4000-0x7FFF, 0xA000-0xBFFF.
    ///
    /// 0b0 = BANK2 affects only accesses to 0x4000-0x7FFF.
    mode: bool,

    /// **Real Time Clock (RTC)**
    ///
    /// $08  RTC S   Seconds   0-59 ($00-$3B)
    ///
    /// $09  RTC M   Minutes   0-59 ($00-$3B)
    ///
    /// $0A  RTC H   Hours     0-23 ($00-$17)
    ///
    /// $0B  RTC DL  Lower 8 bits of Day Counter ($00-$FF)
    ///
    /// $0C  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
    /// > Bit 0  Most significant bit of Day Counter (Bit 8)
    ///
    /// > Bit 6  Halt (0=Active, 1=Stop Timer)
    ///
    /// > Bit 7  Day Counter Carry Bit (1=Counter Overflow)
    rtc: [u8; 5],

    rom_bank_count: i32,
    ram_bank_count: i32,

    rom_mask: usize,
    ram_mask: usize,
}

impl MBC3 {
    /*pub*/
    pub fn new(ch: &CartridgeHeader) -> Self {
        println!("CH: {:?}", ch);
        MBC3 {
            rom: vec![0; ch.rom_size as usize * 1024], // rom_size * 320000?
            ram: vec![0xFF; ch.ram_size as usize * 1024], // ram_size * 1024?

            ramg: 0,
            bank1: 1,
            bank2: 0,
            mode: false,

            rtc: [0; 5],

            rom_bank_count: ch.rom_bank_count,
            ram_bank_count: ch.ram_bank_count,

            rom_mask: MBC3::get_bit_mask(ch.rom_bank_count as usize),
            ram_mask: MBC3::get_bit_mask(ch.ram_bank_count as usize),
        }
    }
}

#[typetag::serde]
impl MBC for MBC3 {
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
                match self.bank2 {
                    0x08..=0x0C => {
                        // RTC
                        self.rtc[(self.bank2 - 0x08) as usize]
                    }
                    _ => {
                        // RAM
                        if self.is_ram_rtc_accessible() {
                            let bank_no = if self.mode { self.bank2 & 0b11 } else { 0 };
                            self.ram_read_byte(bank_no as usize, (addr - 0xA000) as usize)
                        } else {
                            0xFF
                        }
                    }
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
                // Selects the lower 7 bits of ROM bank number
                self.bank1 = byte & 0b111_1111;
            }
            0x4000..=0x5FFF => {
                // RAM Bank Index or RTC Index
                self.bank2 = byte;
            }
            0x6000..=0x7FFF => {
                // ROM/RAM Mode
                // Mode 0 - ROM Banking Mode (default)
                //  > Only RAM bank 0 can be accessed in this mode
                // Mode 1 - RAM Banking Mode
                //  > Only ROM banks 0x01-0x1F can be accessed in this mode
                //  > Other ROM banks will be changed to their corresponding in 0x01-0x1F by clearing the upper 2 bits
                let previous = self.mode;
                self.mode = byte & 0b1 != 0;

                if !previous & self.mode {
                    // 0x00 -> 0x01
                    // Update RTC
                    self.update_rtc();
                }
            }
            0xA000..=0xBFFF => {
                // RAM Bank 00-03 (if any)
                if self.is_ram_rtc_accessible() {
                    let bank_no = if self.mode { self.bank2 } else { 0 };
                    self.ram_write_byte(bank_no as usize, (addr - 0xA000) as usize, byte);
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

impl MBC3 {
    /// Form bit mask depending on the amount of banks for the ROM/RAM
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

    /// Update internal RTC
    fn update_rtc(&mut self) {}

    /// RAM & RTC are only accessible when RAMG's lower nibble is 0xA and there is RAM
    fn is_ram_rtc_accessible(&self) -> bool {
        self.ramg == 0b1010 && self.ram_bank_count > 0
    }

    // ROM R/W
    fn rom_read_byte(&self, bank_no: usize, offset: usize) -> u8 {
        self.rom[self.get_rom_address(bank_no, offset)]
    }

    fn rom_write_byte(&mut self, bank_no: usize, offset: usize, byte: u8) {
        let addr = self.get_rom_address(bank_no, offset);
        self.rom[addr] = byte;
    }

    fn get_rom_address(&self, bank_no: usize, offset: usize) -> usize {
        let real_bank_no: usize = bank_no & self.rom_mask;
        real_bank_no * ROM_BANK_SIZE + offset
    }

    // RAM R/W
    fn ram_read_byte(&self, bank_no: usize, offset: usize) -> u8 {
        self.ram[self.get_ram_address(offset)]
    }

    fn ram_write_byte(&mut self, bank_no: usize, offset: usize, byte: u8) {
        let addr = self.get_ram_address(offset);
        self.ram[addr] = byte;
    }

    fn get_ram_address(&self, offset: usize) -> usize {
        let bank_no: usize = (if self.mode {
            self.bank2 as usize & self.ram_mask
        } else {
            0
        }) as usize;
        let real_offset = offset % (self.ram.len() + 1);

        bank_no * RAM_BANK_SIZE + real_offset
    }
}

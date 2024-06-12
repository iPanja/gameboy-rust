use std::fmt;

const CARTRIDGE_HEADER_SIZE: usize = 0x014F - 0x0100;
const HEADER_START: usize = 0x0100;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type_code: u8,
    pub rom_size_code: u8,
    pub ram_size_code: u8,

    /// ROM Size in KiB (8 => 8KiB)
    pub rom_size: i32,
    /// Amount of ROM banks needed
    pub rom_bank_count: i32,

    /// RAM Size in KiB (8 => 8KiB)
    pub ram_size: i32,
    /// Amount of RAM banks needed
    pub ram_bank_count: i32,
}

impl CartridgeHeader {
    pub fn new(header_bytes: &[u8]) -> Self {
        // [u8; CARTRIDGE_HEADER_SIZE]
        let cartridge_code: u8 = header_bytes[0x0147 - HEADER_START];
        let rom_code: u8 = header_bytes[0x0148 - HEADER_START];
        let ram_code: u8 = header_bytes[0x0149 - HEADER_START];

        let mut title: Vec<char> = Vec::with_capacity(16);
        for index in 0..=(0x0143 - 0x0134) {
            let c_byte = header_bytes[0x0134 - HEADER_START + index];
            let c = char::from(c_byte);
            if c == '\0' {
                break;
            }

            title.push(c);
        }

        let rom_size = (1 << rom_code as usize) * 32;
        let rom_banks = rom_size / 16;

        let (ram_size, ram_banks) = match ram_code {
            0x00 => (0, 0),
            0x01 => (0, 0),
            0x02 => (8, 1),
            0x03 => (32, 4),
            0x04 => (128, 16),
            0x05 => (64, 8),
            _ => (0, 0),
        };

        CartridgeHeader {
            title: title.iter().collect(),
            cartridge_type_code: cartridge_code,
            rom_size_code: rom_code,
            ram_size_code: ram_code,

            rom_size: rom_size,
            rom_bank_count: rom_banks,

            ram_size: ram_size,
            ram_bank_count: ram_banks,
        }
    }
}

impl fmt::Debug for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rom_size = (1 << self.rom_size_code as usize) * 32;
        let rom_banks = rom_size / 16;
        write!(
            f,
            "Cartridge Title: {:?}\nCartridge Code: {:?}\nROM Code: {:?}\n\tSize: {:?}\n\tBanks: {:?}\nRAM Code: {:?}\n\tSize:{:?}\n\tBanks:{:?}",
            self.title, self.cartridge_type_code, self.rom_size_code, rom_size, rom_banks, self.ram_size_code, self.ram_size, self.ram_bank_count
        )
    }
}

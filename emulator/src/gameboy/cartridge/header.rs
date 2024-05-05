const CARTRIDGE_HEADER_SIZE: usize = 0x014F - 0x0100;
const HEADER_START: usize = 0x0100;

pub struct CartridgeHeader {
    title: String,
    cartridge_type_code: u8,
    rom_size_code: u8,
    ram_size_code: u8,
}

impl CartridgeHeader {
    pub fn new(header_bytes: [u8; CARTRIDGE_HEADER_SIZE]) -> Self {
        let cartridge_code: u8 = header_bytes[0x0147 - HEADER_START];
        let rom_code: u8 = header_bytes[0x0148 - HEADER_START];
        let ram_code: u8 = header_bytes[0x0149 - HEADER_START];

        let mut title: Vec<char> = Vec::with_capacity(16);
        for index in 0..=(0x0143 - 0x0134) {
            let c_byte = header_bytes[0x0143 - HEADER_START + index];
            let c = char::from(c_byte);
            title.push(c);
        }

        CartridgeHeader {
            title: title.iter().collect(),
            cartridge_type_code: cartridge_code,
            rom_size_code: rom_code,
            ram_size_code: ram_code,
        }
    }
}

pub trait MBC {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, byte: u8);
    fn load_rom(&mut self, rom_data: &[u8]);
}

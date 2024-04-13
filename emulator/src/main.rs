mod gameboy;

use gameboy::{GameBoy, CPU};
use std::fs::File;
use std::io::Read;

fn main() {
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();

    let mut bootstrap_rom = File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = File::open("../roms/individual/01-special.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&bootstrap_buffer);
    gameboy.read_rom_at(&rom_buffer, 0x101);

    // Simulate ticks
    loop {
        gameboy.tick();
    }
}

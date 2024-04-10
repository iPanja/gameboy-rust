mod gameboy;

use std::fs::File;
use std::io::Read;

use gameboy::{GameBoy, CPU};

fn main() {
    // Read boot ROM file
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    file.read_to_end(&mut buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&buffer);

    // Simulate ticks
    for _ in 0..=255 {
        gameboy.tick();
    }
}

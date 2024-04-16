use std::cmp::min;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{ppu::Pixel, Bus, CPU, PPU};

pub struct GameBoy {
    pub cpu: CPU,
    pub bus: Bus,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn tick(&mut self) {
        let _cycles = self.cpu.step(&mut self.bus);
    }

    pub fn read_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_rom(buffer, 0x0);
    }

    pub fn read_rom_at(&mut self, buffer: &Vec<u8>, addr: usize) {
        self.bus.ram_load_rom(buffer, addr);
    }

    pub fn enable_display(&mut self) {
        self.bus.ram.ram[0xFF40] = 0b1010000;
        self.bus.ram.ram[0xFF42] = 0;
    }

    pub fn get_display(&mut self) -> [[crate::Pixel; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        self.bus.ram.ram[0xFF44] = 0x90; //min(self.bus.ram.ram[0xFF40] + 1, 144);

        self.bus.ppu.get_display(
            &self.bus,
            self.bus.ram_read_byte(0xFF43),
            self.bus.ram_read_byte(0xFF42),
        )
    }
}

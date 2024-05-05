use crate::{DEBUGGER_SCREEN_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{ppu::Pixel, Bus, CPU, PPU};

pub struct GameBoy {
    pub cpu: CPU,
    pub bus: Box<Bus>,
    tile_map_screen: [[Pixel; 16 * 8]; 32 * 8],
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Box::new(Bus::new()),
            tile_map_screen: [[Pixel::Zero; 128]; 256],
        }
    }

    //
    // Stepping/Ticking the CPU
    //
    pub fn tick(&mut self) {
        self.tick_bp(None);
    }

    pub fn tick_bp(&mut self, _breakpoints: Option<&Vec<u16>>) -> bool {
        let mut current_frame_cycles: f64 = 0f64;

        let are_breakpoints_enabled: bool = _breakpoints.is_some();

        while current_frame_cycles < self.bus.timer.get_clock_freq() {
            current_frame_cycles += self.step();
            if are_breakpoints_enabled && _breakpoints.unwrap().contains(&self.cpu.registers.pc) {
                return true;
            }
        }

        return false;
    }

    pub fn step(&mut self) -> f64 {
        let _cycles = self.cpu.tick(&mut self.bus);
        self.bus.tick(_cycles);
        // TODO: TICK PPU

        self.bus.timer.raise_interrupt = match self.bus.timer.raise_interrupt {
            None => None,
            Some(x) => {
                self.bus.trigger_interrupt(x);
                None
            }
        };

        _cycles as f64
    }

    //
    // Reading in ROMs
    //
    pub fn read_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_rom(buffer, 0x0);
    }

    pub fn read_rom_at(&mut self, buffer: &Vec<u8>, addr: usize) {
        self.bus.ram_load_rom(buffer, addr);
    }

    pub fn read_boot_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_boot_rom(buffer);
    }

    //
    // Public display methods
    //
    pub fn enable_display(&mut self) {
        panic!("deprec");
        //self.bus.ram_write_byte(0xFF44, 0x90); //[0xFF44] = 0x90;
        //self.bus.ram_write_byte(0xFF40, 0b1010000); //[0xFF40] = 0b1010000;
        //self.bus.ram_write_byte(0xFF42, 0); //[0xFF42] = 0;
    }

    pub fn export_display(&mut self, buffer: &mut Vec<u8>) {
        *buffer = self.bus.ppu.get_display().to_vec();
    }

    pub fn export_tile_map_display(&mut self, buffer: &mut Vec<u8>) {
        // Update internal frame buffer
        //self.bus.ram_write_byte(0xFF44, 0x90);
        self.bus.ppu.get_debug_display(&mut self.tile_map_screen);

        // Export as vector
        self.convert_disply_to_vec(&self.tile_map_screen, buffer);
    }

    //
    // Display helper methods
    //

    fn get_debug_display(&mut self) -> &[[Pixel; 128]; 256] {
        //self.bus.ram.ram[0xFF44] = 0x90; //min(self.bus.ram.ram[0xFF40] + 1, 144);
        self.bus.ram_write_byte(0xFF44, 0x90);
        self.bus.ppu.get_debug_display(&mut self.tile_map_screen);

        &self.tile_map_screen
    }

    //pub fn convert(&mut self, display: , buffer: &mut Vec<u8>)
    fn convert_disply_to_vec<const W: usize, const H: usize>(
        &self,
        display: &[[Pixel; W]; H],
        buffer: &mut Vec<u8>,
    ) {
        *buffer = Vec::with_capacity(W * H);
        let pallete = self.bus.ppu.bg_palette;

        for row in display {
            for pixel in row {
                let bits = match pixel {
                    Pixel::Three => (pallete >> 6) & 0x3,
                    Pixel::Two => (pallete >> 4) & 0x3,
                    Pixel::One => (pallete >> 2) & 0x3,
                    Pixel::Zero => (pallete >> 0) & 0x3,
                };
                let color = sdl2::pixels::Color::from(Pixel::from(bits as u8));

                buffer.push(color.r);
                buffer.push(color.g);
                buffer.push(color.b);
            }
        }
    }
}

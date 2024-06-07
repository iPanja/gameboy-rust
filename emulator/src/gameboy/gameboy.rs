use super::{joypad::JoypadInputKey, ppu::Pixel, Bus, CartridgeHeader, CPU, PPU};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameBoy {
    pub cpu: CPU,
    pub bus: Box<Bus>,
    pub cartridge_header: Option<CartridgeHeader>,
    //tile_map_screen: [[Pixel; 16 * 8]; 32 * 8],
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Box::new(Bus::new()),
            cartridge_header: None,
            //tile_map_screen: [[Pixel::Zero; 128]; 256],
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
        let _cycles = self.cpu.tick(&mut self.bus) as f64;
        self.bus.tick(_cycles);

        self.bus.timer.raise_interrupt = match self.bus.timer.raise_interrupt {
            None => None,
            Some(x) => {
                self.bus.trigger_interrupt(x);
                None
            }
        };

        self.bus.joypad.read_byte();
        self.bus.joypad.raise_interrupt = match self.bus.joypad.raise_interrupt {
            None => None,
            Some(x) => {
                self.bus.trigger_interrupt(x);
                None
            }
        };

        _cycles
    }

    //
    // Reading in ROMs
    //
    pub fn read_rom(&mut self, buffer: &Vec<u8>) {
        self.cartridge_header = Some(CartridgeHeader::new(&buffer[0x0100..=0x014F]));
        // CAUSES NINTENDO LOGO TO DISAPPEAR??

        if let Some(c_h) = &self.cartridge_header {
            match c_h.cartridge_type_code {
                0x00 => self.bus.mbc = Box::new(super::cartridge::MBC0::new()),
                0x01 | 0x02 | 0x03 => self.bus.mbc = Box::new(super::cartridge::MBC1::new(c_h)),
                _ => {
                    panic!("Unsupported cartridge!\n\t{:#X}\n", c_h.cartridge_type_code);
                    //self.bus.mbc = Box::new(super::cartridge::MBC0::new())
                }
            }
        }

        self.bus.ram_load_rom(buffer, 0x0);
    }

    pub fn read_boot_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_boot_rom(buffer);
    }

    //
    // Public display methods
    //

    pub fn export_display(&mut self, buffer: &mut Vec<u8>) {
        *buffer = self.bus.ppu.get_display().to_vec();
    }

    pub fn export_tile_map_display(&mut self, buffer: &mut Vec<u8>) {
        // Update internal frame buffer
        let mut tile_map_screen = [[Pixel::Zero; 128]; 256];
        self.bus.ppu.get_debug_display(&mut tile_map_screen);

        // Export as vector
        self.convert_disply_to_vec(&tile_map_screen, buffer);
    }

    //
    // Display helper methods
    //

    /*fn get_debug_display(&mut self) -> &[[Pixel; 128]; 256] {
        //self.bus.ram.ram[0xFF44] = 0x90; //min(self.bus.ram.ram[0xFF40] + 1, 144);
        self.bus.ram_write_byte(0xFF44, 0x90);
        self.bus.ppu.get_debug_display(&mut self.tile_map_screen);

        &self.tile_map_screen
    }*/

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
                let gray_value = (Pixel::from(bits as u8)).rgb_value();

                buffer.push(gray_value);
                buffer.push(gray_value);
                buffer.push(gray_value);
                buffer.push(0xFF);
            }
        }
    }
}

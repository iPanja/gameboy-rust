use super::{joypad::JoypadInputKey, ppu::Pixel, Bus, CartridgeHeader, CPU, PPU};

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
        }
    }

    //
    // Stepping/Ticking the CPU
    //

    /// Execute one tick's worth of opcodes
    pub fn tick(&mut self) {
        self.tick_bp(None);
    }

    /// Execute one tick's worth of opcodes, returning early if a breakpoint is encountered
    ///
    /// Returns whether or not a breakpoint was hit
    pub fn tick_bp(&mut self, _breakpoints: Option<&Vec<u16>>) -> bool {
        let mut current_frame_cycles: f64 = 0f64;

        let are_breakpoints_enabled: bool = _breakpoints.is_some();

        while current_frame_cycles < self.bus.timer.get_clock_freq() {
            current_frame_cycles += self.step() as f64;
            if are_breakpoints_enabled && _breakpoints.unwrap().contains(&self.cpu.registers.pc) {
                return true;
            }
        }

        return false;
    }

    /// Execute a single opcode
    pub fn step(&mut self) -> u8 {
        // Execute one CPU instruction
        let _cycles: u8 = self.cpu.step(&mut self.bus);
        // Tick appropriate components through the bus
        self.bus.tick(_cycles);

        _cycles
    }

    //
    // Reading in ROMs
    //

    /// Load the supplied ROM's buffer
    ///
    /// It will parse the ROM's cartridge header and load the appropriate MBC
    pub fn read_rom(&mut self, buffer: &Vec<u8>) {
        self.cartridge_header = Some(CartridgeHeader::new(&buffer[0x0100..=0x014F]));
        // CAUSES NINTENDO LOGO TO DISAPPEAR??

        if let Some(c_h) = &self.cartridge_header {
            match c_h.cartridge_type_code {
                0x00 => self.bus.mbc = Box::new(super::cartridge::MBC0::new()),
                0x01 | 0x02 | 0x03 => self.bus.mbc = Box::new(super::cartridge::MBC1::new(c_h)),
                0x0F | 0x10 | 0x11 | 0x12 | 0x13 => {
                    self.bus.mbc = Box::new(super::cartridge::MBC3::new(c_h))
                }
                _ => {
                    panic!("Unsupported cartridge!\n\t{:#X}\n", c_h.cartridge_type_code);
                }
            }
        }

        self.bus.ram_load_rom(buffer, 0x0);
    }

    /// Load the Boot ROM into memory (0x0000-0x0100)
    pub fn read_boot_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_boot_rom(buffer);
    }

    //
    // Public display methods
    //

    /// Copy the display buffer into the buffer supplied
    pub fn export_display(&mut self, buffer: &mut Vec<u8>) {
        *buffer = self.bus.ppu.get_display().to_vec();
    }

    /// Copy the PPU's internal tile map display into the buffer specified
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

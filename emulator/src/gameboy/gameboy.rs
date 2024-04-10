use super::{Bus, CPU};

pub struct GameBoy {
    pub cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn tick(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn read_rom(&mut self, buffer: &Vec<u8>) {
        self.bus.ram_load_rom(buffer);
    }
}

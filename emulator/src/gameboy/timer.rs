use super::{Bus, Interrupt};

const CPU_CLOCK: f64 = 4194304f64; // Random value, fix this

pub struct Timer {
    // External Memory Mapped
    div: u8, // Divider register - 0xFF04  |   These are the upper 8 bits that compose the internal div
    tima: u8, // Timer counter - 0xFF05
    tma: u8, // Timer modulo - 0xFF06
    tac: u8, // Timer control - 0xFF07
    // Internal Registers
    internal_div: u16,
    prev_and_result: bool,
    pending_tma_reset: u8,
    // Raise timer interrupt
    pub raise_interrupt: Option<Interrupt>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            internal_div: 0,
            prev_and_result: false,
            pending_tma_reset: 0,
            raise_interrupt: None,
        }
    }

    pub fn read_byte(&self, index: usize) -> u8 {
        match index {
            0x00 => self.div,
            0x01 => self.tima,
            0x02 => self.tma,
            0x03 => self.tac,
            _ => panic!(
                "Timer should not be requesting to read an offset of: {:#X}",
                index
            ),
        }
    }

    pub fn write_byte(&mut self, index: usize, value: u8) {
        match index {
            0x00 => self.internal_div = 0x00, // Writing any value to this register resets it to 0x00
            0x01 => {
                self.tima = value;
                self.pending_tma_reset = 0
            }
            0x02 => self.tma = value,
            0x03 => self.tac = value,
            _ => panic!(
                "Timer should not be requesting to read an offset of: {:#X}",
                index
            ),
        };
    }

    pub fn tick(&mut self, t_cycles: u8) {
        self.internal_div = self.internal_div.wrapping_add(t_cycles as u16);

        self.pending_tma_reset = match self.pending_tma_reset {
            1 => {
                self.tima = self.tma;
                self.raise_interrupt = Some(Interrupt::Timer);
                0
            }
            0 => 0,
            x => x - 1,
        };

        let bit_pos = match self.tac & 0x3 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => panic!("Unsupported??? {:#X}", self.tac & 0x3),
        };
        let bit = self.internal_div & (0b1 << bit_pos);

        let and_result = bit != 0 && self.is_clock_enabled();
        if !self.prev_and_result && and_result {
            // Looking for falling edge (1 -> 0)
            self.tima = self.tima.wrapping_add(1);
            // Delay overflow checking
            if self.tima == 0x00 {
                self.pending_tma_reset = 5;
            }
        }
    }

    pub fn get_clock_freq(&self) -> f64 {
        CPU_CLOCK
            / match self.tac & 0x3 {
                0b00 => 1024, // Frequency 4096
                0b01 => 16,   // Frqeuency 262144
                0b10 => 64,   // Frequency 65536
                0b11 => 256,  // Frequency 16382
                _ => panic!("Invalid clock freq!"),
            } as f64
    }

    fn is_clock_enabled(&self) -> bool {
        return self.tac & 0x4 != 0;
    }
}

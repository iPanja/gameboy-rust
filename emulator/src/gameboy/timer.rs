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
    internal_tima: u16,
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
            internal_tima: 0,
            prev_and_result: false,
            pending_tma_reset: 0,
            raise_interrupt: None,
        }
    }

    pub fn read_byte(&self, index: usize) -> u8 {
        match index {
            //0x00 => ((self.internal_div & 0xFF00) >> 8) as u8,
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
            //0x00 => self.internal_div = 0x00, // Writing any value to this register resets it to 0x00
            0x00 => self.div = 0,
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

    pub fn tick(&mut self, t_cycles: f64) {
        if t_cycles == 0.0 {
            return;
        }

        // 1 tick @ 16384 Hz = 256 cycles (markau)
        self.internal_div += t_cycles as u16;
        while self.internal_div >= 256 {
            self.div = self.div.wrapping_add(1);
            self.internal_div -= 256;
        }

        // Increment TIMA, handle overflow (triggers an interrupt)
        if self.is_timer_enabled() {
            self.internal_tima += t_cycles as u16;
            let clock_select = self.get_clock_select();

            while self.internal_tima >= clock_select {
                self.internal_tima -= clock_select;
                self.tima = self.tima.wrapping_add(1);

                if self.tima == 0x00 {
                    // TIMA overflow
                    self.raise_interrupt = Some(Interrupt::Timer);
                    self.tima = self.tma;
                }
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

    pub fn get_clock_select(&self) -> u16 {
        match self.tac & 0x3 {
            0b00 => 1024, // Frequency 4096
            0b01 => 16,   // Frqeuency 262144
            0b10 => 64,   // Frequency 65536
            0b11 => 256,  // Frequency 16382
            _ => panic!("Invalid clock freq!"),
        }
    }

    fn is_timer_enabled(&self) -> bool {
        return self.tac & 0x4 != 0;
    }
}

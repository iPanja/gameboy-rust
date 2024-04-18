use super::{Bus, Interrupt};

pub struct Timer {
    div: u8,  // Divider register - 0xFF04
    tima: u8, // Timer counter - 0xFF05
    tma: u8,  // Timer modulo - 0xFF06
    tac: u8,  // Timer control - 0xFF07
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
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
            0x00 => self.div = 0x00, // Writing any value to this register resets it to 0x00
            0x01 => self.tima = value,
            0x02 => self.tma = value,
            0x03 => self.tac = value,
            _ => panic!(
                "Timer should not be requesting to read an offset of: {:#X}",
                index
            ),
        };
    }

    pub fn increment_clock(&mut self, bus: &mut Bus, freq: u8) {
        // TODO - If a TMA write is executed on the same M-cycle as the content of TMA is transferred to TIMA due to a timer overflow, the old value is transferred to TIMA.
        let result = self.tima.wrapping_add(freq);

        self.tima = if result < self.tima {
            // Overflow!
            bus.trigger_interrupt(Interrupt::Timer);
            self.tma
        } else {
            result
        }
    }

    fn get_clock_freq(&self) -> u16 {
        match self.tac & 0x3 {
            0b00 => 256,
            0b01 => 4,
            0b10 => 16,
            0b11 => 64,
            _ => panic!("Invalid clock freq!"),
        }
    }

    fn is_tima_enabled(&self) -> bool {
        return self.tac & 0x4 != 0;
    }
}

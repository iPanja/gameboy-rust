use super::FlagsRegister;
use std::fmt;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
                a: {:#X}\t f: {:#X}
                b: {:#X}\t c: {:#X}
                d: {:#X}\t e: {:#X}
                h: {:#X}\t l: {:#X}\n
                pc: {:#X}
                sp: {:#X}
            ",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp
        )
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0x01,
            b: 0x00,
            d: 0x00,
            h: 0x01,
            f: FlagsRegister::from(0xB0),
            c: 0x13,
            e: 0xD8,
            l: 0x4D,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    // 16-bit register helper methods
    /// Getters
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    // Setters
    pub fn set_af(&mut self, bytes: u16) {
        self.a = (bytes >> 8) as u8;
        self.f = FlagsRegister::from((bytes & 0xFF) as u8);
    }

    pub fn set_bc(&mut self, bytes: u16) {
        self.b = (bytes >> 8) as u8;
        self.c = (bytes & 0xFF) as u8;
    }

    pub fn set_de(&mut self, bytes: u16) {
        self.d = (bytes >> 8) as u8;
        self.e = (bytes & 0xFF) as u8;
    }

    pub fn set_hl(&mut self, bytes: u16) {
        self.h = (bytes >> 8) as u8;
        self.l = (bytes & 0xFF) as u8;
    }

    // Modifiers
    pub fn hli(&mut self) -> u16 {
        let value = self.get_hl();
        self.set_hl(value + 1);

        value
    }

    pub fn hld(&mut self) -> u16 {
        let value = self.get_hl();
        self.set_hl(value - 1);

        value
    }
}

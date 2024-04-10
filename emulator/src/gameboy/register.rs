use super::FlagsRegister;
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    f: FlagsRegister,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            b: 0,
            d: 0,
            h: 0,
            f: FlagsRegister::new(),
            c: 0,
            e: 0,
            l: 0,
            pc: 0,
            sp: 0,
        }
    }

    // 16-bit register helper methods
    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_af(&mut self, bytes: u16) {
        self.a = (bytes >> 8) as u8;
        self.f = FlagsRegister::from((bytes & 0xFF) as u8);
    }

    fn set_bc(&mut self, bytes: u16) {
        self.b = (bytes >> 8) as u8;
        self.c = (bytes & 0xFF) as u8;
    }

    fn set_de(&mut self, bytes: u16) {
        self.d = (bytes >> 8) as u8;
        self.e = (bytes & 0xFF) as u8;
    }

    fn set_hl(&mut self, bytes: u16) {
        self.h = (bytes >> 8) as u8;
        self.l = (bytes & 0xFF) as u8;
    }
}

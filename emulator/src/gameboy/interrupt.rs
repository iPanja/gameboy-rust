#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum Interrupt {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    pub fn get_flag_mask(&self) -> u8 {
        match self {
            Interrupt::VBlank => 0x1,
            Interrupt::LCD => 0x2,
            Interrupt::Timer => 0x4,
            Interrupt::Serial => 0x8,
            Interrupt::Joypad => 0x10,
        }
    }

    pub fn get_jump_addr(&self) -> u8 {
        match self {
            Interrupt::VBlank => 0x40,
            Interrupt::LCD => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Joypad => 0x60,
        }
    }
}

impl From<Interrupt> for u8 {
    fn from(interrupt: Interrupt) -> u8 {
        interrupt.get_flag_mask()
    }
}

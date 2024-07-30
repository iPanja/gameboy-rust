#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub enum Interrupt {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    /// Get the bits used in the IE, IF Reg to signify the interrupt
    pub fn get_flag_mask(&self) -> u8 {
        match self {
            Interrupt::VBlank => 0x1,
            Interrupt::LCD => 0x2,
            Interrupt::Timer => 0x4,
            Interrupt::Serial => 0x8,
            Interrupt::Joypad => 0x10,
        }
    }

    /// Get the PC address of the respective interrupt handler
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

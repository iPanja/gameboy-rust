pub enum JoypadInputKey {
    Start,
    Select,
    B,
    A,
    Down,
    Up,
    Left,
    Right,
}

impl JoypadInputKey {
    pub fn parse(key: imgui::Key) -> Option<JoypadInputKey> {
        match key {
            imgui::Key::F2 => Some(JoypadInputKey::Start),
            imgui::Key::F1 => Some(JoypadInputKey::Select),
            imgui::Key::E => Some(JoypadInputKey::A),
            imgui::Key::Q => Some(JoypadInputKey::B),
            imgui::Key::W => Some(JoypadInputKey::Up),
            imgui::Key::A => Some(JoypadInputKey::Left),
            imgui::Key::S => Some(JoypadInputKey::Down),
            imgui::Key::D => Some(JoypadInputKey::Right),
            _ => None,
        }
    }

    pub fn input_byte_pos(&self) -> u8 {
        match self {
            JoypadInputKey::Start => 0b1000_0000,
            JoypadInputKey::Select => 0b0100_0000,
            JoypadInputKey::B => 0b0010_0000,
            JoypadInputKey::A => 0b0001_0000,
            JoypadInputKey::Down => 0b0000_1000,
            JoypadInputKey::Up => 0b0000_0100,
            JoypadInputKey::Left => 0b0000_0010,
            JoypadInputKey::Right => 0b0000_0001,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
enum JoypadSelectionMask {
    Buttons,
    Directions,
}

pub struct Joypad {
    input_byte: u8, // Start | Select | B | A | Down | Up | Left | Right
    selection_mask: JoypadSelectionMask,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            input_byte: 0xFF, // Start as all unpressed
            selection_mask: JoypadSelectionMask::Buttons,
        }
    }

    pub fn read_byte(&self) -> u8 {
        // Reading 0xFF00
        match self.selection_mask {
            JoypadSelectionMask::Buttons => self.input_byte >> 4,
            JoypadSelectionMask::Directions => self.input_byte & 0x0F,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte & 0b0011_0000 >> 4 {
            0b10 => self.selection_mask = JoypadSelectionMask::Buttons,
            0b01 => self.selection_mask = JoypadSelectionMask::Directions,
            _ => (),
        }
    }

    pub fn press_key(&mut self, joypad_key: JoypadInputKey) {
        let bit = joypad_key.input_byte_pos();
        self.input_byte &= !bit; // Clearing bit - 0 = pressed
    }

    pub fn press_key_raw(&mut self, key: imgui::Key) {
        if let Some(joypad_input) = JoypadInputKey::parse(key) {
            self.press_key(joypad_input);
        }
    }

    pub fn unpress_key(&mut self, joypad_key: JoypadInputKey) {
        let bit = joypad_key.input_byte_pos();
        self.input_byte |= bit; // Setting bit - 1 = unpressed
    }

    pub fn unpress_key_raw(&mut self, key: imgui::Key) {
        if let Some(joypad_input) = JoypadInputKey::parse(key) {
            self.unpress_key(joypad_input);
        }
    }
}

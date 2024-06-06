use std::slice::Iter;

use serde::{Deserialize, Serialize};

use super::Interrupt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn iter() -> Iter<'static, JoypadInputKey> {
        static JOYPAD_KEYS: [JoypadInputKey; 8] = [
            JoypadInputKey::Start,
            JoypadInputKey::Select,
            JoypadInputKey::B,
            JoypadInputKey::A,
            JoypadInputKey::Down,
            JoypadInputKey::Up,
            JoypadInputKey::Left,
            JoypadInputKey::Right,
        ];
        JOYPAD_KEYS.iter()
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

pub struct Joypad {
    pub input_byte: u8,     // Start | Select | B | A | Down | Up | Left | Right
    pub selection_mask: u8, // 2 bits
    pub raise_interrupt: Option<Interrupt>,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            input_byte: 0xFF, // Start as all unpressed
            selection_mask: 0,
            raise_interrupt: None,
        }
    }

    pub fn read_byte(&self) -> u8 {
        // Reading 0xFF00
        let mut result: u8 = 0xF0;

        if self.selection_mask == 0b10 {
            // Read Directions; lower nibble
            self.input_byte & 0xF | 0xF0;
            result |= self.input_byte & 0x0F;
        }
        if self.selection_mask == 0b01 {
            // Read Buttons; upper nible
            result |= (self.input_byte & 0xF0) >> 4;
        }
        if self.selection_mask == 0b11 {
            // ...
            result |= 0xF;
        }

        result
    }

    pub fn write_byte(&mut self, byte: u8) {
        // Update reading mode
        self.selection_mask = (byte & 0x30) >> 4;

        /*println!(
            "joypad mode: {:?}\t({:08b})",
            self.selection_mask,
            byte & 0b0011_0000 >> 4
        );*/
    }

    pub fn press_key(&mut self, joypad_key: JoypadInputKey) {
        let bit = joypad_key.input_byte_pos();

        let prev_value = self.input_byte & bit;
        self.input_byte &= !bit; // Clearing bit; 0 = pressed

        // If this bit is currently monitored, trigger an interrupt
        let should_trigger = {
            if self.selection_mask == 0b10 {
                // Directions
                prev_value == 1 && bit < 0x10
            } else if self.selection_mask == 0b01 {
                // Buttons
                prev_value == 1 && bit > 0xF
            } else {
                false
            }
        };

        if should_trigger {
            self.raise_interrupt = Some(Interrupt::Joypad);
        }
    }

    pub fn unpress_key(&mut self, joypad_key: JoypadInputKey) {
        let bit = joypad_key.input_byte_pos();
        self.input_byte |= bit; // Setting bit; 1 = unpressed
    }
}

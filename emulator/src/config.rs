use std::{collections::HashMap, path::PathBuf};

use winit::event::VirtualKeyCode;

use crate::gameboy::joypad::JoypadInputKey;

#[derive(Clone)]
pub struct GameBoyConfig {
    pub selected_rom: Option<PathBuf>,
    pub input_mapper: HashMap<JoypadInputKey, [Option<VirtualKeyCode>; 2]>,
}

impl Default for GameBoyConfig {
    fn default() -> Self {
        GameBoyConfig {
            selected_rom: None,
            /*input_mapper: HashMap::from([
                (VirtualKeyCode::W, JoypadInputKey::Up),
                (VirtualKeyCode::A, JoypadInputKey::Left),
                (VirtualKeyCode::S, JoypadInputKey::Down),
                (VirtualKeyCode::D, JoypadInputKey::Right),
                (VirtualKeyCode::Right, JoypadInputKey::A),
                (VirtualKeyCode::Left, JoypadInputKey::B),
                (VirtualKeyCode::Q, JoypadInputKey::Start),
                (VirtualKeyCode::E, JoypadInputKey::Select),
            ]),*/
            input_mapper: HashMap::from([
                (JoypadInputKey::Up, [Some(VirtualKeyCode::W), None]),
                (JoypadInputKey::Left, [Some(VirtualKeyCode::A), None]),
                (JoypadInputKey::Down, [Some(VirtualKeyCode::S), None]),
                (JoypadInputKey::Right, [Some(VirtualKeyCode::D), None]),
                (JoypadInputKey::A, [Some(VirtualKeyCode::Right), None]),
                (JoypadInputKey::B, [Some(VirtualKeyCode::Left), None]),
                (JoypadInputKey::Start, [Some(VirtualKeyCode::Q), None]),
                (JoypadInputKey::Select, [Some(VirtualKeyCode::E), None]),
            ]),
        }
    }
}

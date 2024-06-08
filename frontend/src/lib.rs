const SCALE: u32 = 4;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DEBUGGER_SCREEN_WIDTH: usize = 16 * 8;
const DEBUGGER_SCREEN_HEIGHT: usize = 32 * 8;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const CYCLES_PER_FRAME: f64 = (4194304 / 60) as f64;

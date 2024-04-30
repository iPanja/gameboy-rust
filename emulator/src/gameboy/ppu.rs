use sdl2::pixels::Color;
use std::{
    fmt,
    fs::OpenOptions,
    io::{empty, prelude::*},
};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{Bus, Interrupt};

const VRAM_SIZE: usize = (0x9FFF - 0x8000) + (0xFE9F - 0xFE00);

const TILE_MAP_SIZE: usize = 0x3FF;

pub struct PPU {
    // Memory Map (entirety of VRAM) 0x8000-0x9FFF and 0xFE00-0xFE9F
    raw_tile_vram: [u8; 384 * 16],
    pub tile_set: [Tile; 384],   // Tile Set Blocks 0-3 - 0x8000-0x97FF
    tile_map_1: [u8; 0x3FF + 1], // Background Map 1 - 0x9800 - 0x9BFF    // Each entry (byte, u8) is a tile number (tile located in tile_set)
    tile_map_2: [u8; 0x3FF + 1], // Background Map 2 - 0x9C00 - 0x9FFF    // "                                                                "
    raw_oam: [u8; 0xA0], // Object Attribute Memory - 0xFE00 - 0xFE9F // Each entry is 4 bytes, [u8; 4] - https://gbdev.io/pandocs/OAM.html#object-attribute-memory-oam
    oam: [Sprite; 40],   // [[u8; 4]; 40]
    // IO Registers 0xFF40-0xFF4B
    lcdc: u8,           // PPU control register - 0xFF40
    stat: u8,           // PPU status register - 0xFF41
    scy: u8,            // Vertical scroll register - 0xFF42
    scx: u8,            // Horizontal scroll register - 0xFF43
    pub ly: u8,         // Scanline register - 0xFF44
    lyc: u8,            // LY Compare - 0xFF45
    pub bg_palette: u8, // Background color palette - 0xFF47
    ob_palette_1: u8,   // Object color palette 1 - 0xFF48
    ob_palette_2: u8,   // Object color palette 2 - 0xFF49
    wy: u8,             // Window Y position - 0xFF4A
    wx: u8,             // Window X position - 0xFF4B
    // Internal data structures
    mode_cycles: u16,
    scanline_sprite_cache: Vec<Sprite>,
    // Display
    screen_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3], // RGB
}

// https://github.com/Hacktix/GBEDG/blob/master/ppu/index.md#the-concept-of-ppu-modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    HBlank = 0, // Takes place after the current scanline has completed, pads the duration of the line scan to 456 T-Cycles - effectively a pause for the PPU.
    VBlank = 1, // The psuedo lines (144-153), taking place after the entire screen has been scanned
    OAM = 2, // Entered at the start of every scanline (except for V-Blank), before pixels are actually drawn to the screen. Renders sprites.
    Drawing = 3, // Where the PPU transfers pixels to the LCD.
}

impl std::convert::From<Mode> for u8 {
    fn from(mode: Mode) -> u8 {
        match mode {
            Mode::HBlank => 0b00,
            Mode::VBlank => 0b01,
            Mode::OAM => 0b10,
            Mode::Drawing => 0b11,
        }
    }
}
impl std::convert::From<u8> for Mode {
    fn from(byte: u8) -> Mode {
        match byte {
            0b11 => Mode::Drawing,
            0b10 => Mode::OAM,
            0b01 => Mode::VBlank,
            _ => Mode::HBlank,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)] // Debug
pub enum Pixel {
    Three, // 0b11
    Two,   // 0b10
    One,   // 0b01
    Zero,  // 0b00
}

impl std::convert::From<Pixel> for Color {
    fn from(pixel: Pixel) -> Color {
        match pixel {
            Pixel::Three => Color::RGB(0, 0, 0), // Black
            Pixel::Two => Color::RGB(85, 85, 85),
            Pixel::One => Color::RGB(170, 170, 170),
            Pixel::Zero => Color::RGB(255, 255, 255), // White
        }
    }
}
impl std::convert::From<Pixel> for u8 {
    fn from(pixel: Pixel) -> u8 {
        match pixel {
            Pixel::Three => 0b11,
            Pixel::Two => 0b10,
            Pixel::One => 0b01,
            Pixel::Zero => 0b00,
        }
    }
}
impl std::convert::From<u8> for Pixel {
    fn from(byte: u8) -> Pixel {
        match byte {
            0b11 => Pixel::Three,
            0b10 => Pixel::Two,
            0b01 => Pixel::One,
            _ => Pixel::Zero,
        }
    }
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pixel::Three => write!(f, "W"),
            Pixel::One => write!(f, " "),
            Pixel::Two => write!(f, "+"),
            Pixel::Zero => write!(f, "B"),
        }
    }
}

pub type Tile = [[Pixel; 8]; 8];
fn empty_tile() -> Tile {
    [[Pixel::Zero; 8]; 8]
}

pub type Sprite = [u8; 4];
fn empty_sprite() -> Sprite {
    [0; 4]
}

/* Registers
    0xFF40 - LCDC - PPU control register
    0xFF41 - STAT - PPU status register
    0xFF42 - SCY - Vertical scroll register
    0xFF43 - SCX - Horizontal scroll register
    0xFF44 - LY - Scanline register
*/

/* VRAM Data
    Tile Set Block 0 - $8000-87FF
    Tile Set Block 1 - $8800-8FFF
    Tile Set Block 2 - $9000-97FF

    Tile Map 0 - $9800-9BFF
    Tile Map 1 - $9C00-9FFF
*/

/* Addressing
    > $8000 method
        > $8000 as base pointer
        > unsigned addressing
            > Block 0: 0-127
            > Block 1: 128-255
            > Block 2: ...
    > $8800 method
        > $9000 as base pointer
        > signed addressing
            > Block 2: 0-127
            > tiles -128 to -1 are in Block 1
    > Objects always use $8000 addressing
    > BG, Window can use either mode (determined by LCDC bit 4)
*/

/* Tile
    A tile assigns a color ID to a pixel, ranging from 0-3 (2 bits).
        > 0b11 => white (W)
        > 0b10 => dark-gray (D)
        > 0b01 => light-gray (L)
        > 0b00 => black (B)
    For the Background or Window, the color ID => a pallete
    For Objects, color ID 0 = transparent, 1-3 => a pallete

    Each tile occupies 16 bytes, where each line is represented by 2 bytes.
        > Each pixel requires 2 bits
        > Each tile is 8x8 = 64 pixels
        > 64 pixels * 2bits/pixel = 128 bits or 16 bytes per tile
    Each row of a tile is 2 bytes:
        > The first byte specifies the LSB of the color ID of each pixel
        > The second byte specifies the MSB
        > In both bytes, bit 7 represents the leftmost pixel, and bit 0 the rightmost

    Example: $3C $7E
        > 00111100  01111110
        > 0         0       = 00 => 00 (COLOR ID)
        >  0         1      = 01 => 10 (COLOR ID, d2)
        > Entire row: 0 2 3 3 3 3 2 0
        > so color IDs range from 0-3 (requires only 2 bits)
*/

/* VRAM Tile Maps
   The Game Boy contains two 32x32 tile maps in VRAM:
       > $9800-9BFF
       > $9C00-9FFF
    Any of these maps can be used to display the Background or the Window.
    Each map contains the 1-byte indexes of the tiles to be displayed.
        > Tiles are obtained from the Tile Data Table
*/

/* The Three Layers
    Background:
        > Composed of a tilemap
        > Scrolling performed via ...
    Window:
        > A rectangle canvas/UI layer of sorts
        > No transparency
        > You can only control the position of the top-left pixel of the window
    Objects:
        > "Sprites"
        > Objects are made of 1 or 2 stacked tiles (8x8 or 8x16)
        > Can be displayed anywhere on screen
        > Sometimes many sprites are combined to create larger ones
        > Use the $8000 addressing
*/

/* Object Attribute Memory (OAM) - https://gbdev.io/pandocs/OAM.html
    > Up to 40 movable objects (sprites) can be displayed on the screen
    > Only up to 10 of these can occupy the same scanline
        > Regardless of if they are within the viewport
    > Taken from tile blocks 0 and 1, locatd at $8000-8FFF and have unsigned numbering
    > Object attributes are stored in the OAM at $FE00-FE9F
        > Byte 0 - Y Position
        > Byte 1 - X Position
        > Byte 2 - Tile Index
        > Byte 3 - Attributes/Flags
*/

impl PPU {
    pub fn new() -> Self {
        PPU {
            raw_tile_vram: [0; 384 * 16],
            tile_set: [empty_tile(); 384],
            tile_map_1: [0; 0x3FF + 1],
            tile_map_2: [0; 0x3FF + 1],
            raw_oam: [0; 0xA0],
            oam: [empty_sprite(); 40],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bg_palette: 0,
            ob_palette_1: 0,
            ob_palette_2: 0,
            wy: 0,
            wx: 0,
            screen_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            mode_cycles: 0,
            scanline_sprite_cache: Vec::with_capacity(10),
        }
    }

    pub fn read_byte(&self, index: usize, real_addr: usize) -> u8 {
        match real_addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bg_palette,
            0xFF48 => self.ob_palette_1,
            0xFF49 => self.ob_palette_2,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0x8000..=0x97FF => self.read_tile_set_data_as_byte(real_addr - 0x8000),
            0x9800..=0x9BFF => self.tile_map_1[real_addr - 0x9800],
            0x9C00..=0x9FFF => self.tile_map_2[real_addr - 0x9C00],
            0xFE00..=0xFE9F => self.raw_oam[real_addr - 0xFE00],
            _ => {
                panic!("Unsupported VRAM access at byte: {:#X}", real_addr);
            }
        }
    }

    pub fn write_byte(&mut self, index: usize, real_addr: usize, value: u8) {
        //println!("offset of: {:#X}", index);

        // println!("writing to: {:#X} - {:#X}", real_addr, value);

        match real_addr {
            0xFF40 => {
                self.lcdc = value;
            }
            0xFF41 => {
                self.stat = value;
            }
            0xFF42 => {
                self.scy = value;
            }
            0xFF43 => {
                self.scx = value;
            }
            0xFF44 => {
                self.ly = value;
            }
            0xFF45 => {
                self.lyc = value;
            }
            0xFF47 => {
                self.bg_palette = value;
            }
            0xFF48 => {
                self.ob_palette_1 = value;
            }
            0xFF49 => {
                self.ob_palette_2 = value;
            }
            0xFF4A => {
                self.wy = value;
            }
            0xFF4B => {
                self.wx = value;
            }
            0x8000..=0x97FF => {
                self.write_tile_set_data(real_addr - 0x8000, value);
            }
            0x9800..=0x9BFF => {
                self.tile_map_1[real_addr - 0x9800] = value;
            }
            0x9C00..=0x9FFF => {
                self.tile_map_2[real_addr - 0x9C00] = value;
            }
            0xFE00..=0xFE9F => self.write_oam_data(real_addr - 0xFE00, value),
            _ => {
                panic!("Unsupported VRAM access at byte: {:#X}", real_addr);
            }
        };
    }

    fn get_mode(&self) -> Mode {
        let lower = self.stat & 0b11;
        Mode::from(lower)
    }

    fn set_mode(&mut self, new_mode: Mode) -> Option<Interrupt> {
        self.stat &= !0b11; // Clear mode bits
        self.stat |= u8::from(new_mode); // Set mode bits

        match new_mode {
            Mode::Drawing => None,
            Mode::HBlank => {
                if self.stat & 0b100 != 0 {
                    Some(Interrupt::LCD)
                } else {
                    None
                }
            }
            Mode::VBlank => {
                if self.stat & 0b1000 != 0 {
                    Some(Interrupt::LCD)
                } else {
                    None
                }
            }
            Mode::OAM => {
                if self.stat & 0b1_0000 != 0 {
                    Some(Interrupt::LCD)
                } else {
                    None
                }
            }
        }
    }

    fn write_tile_set_data(&mut self, index: usize, byte: u8) {
        self.raw_tile_vram[index] = byte;
        // A tile is 2 bytes. 1 byte will only populate half the tile, or 4 pixels
        // Cache
        let first_index = index & 0xFFFE;

        let byte1 = self.raw_tile_vram[first_index];
        let byte2 = self.raw_tile_vram[first_index + 1];

        let tile_index = index / 16;
        let row_index = (index % 16) / 2;
        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;
            let value = match (lsb != 0, msb != 0) {
                (true, true) => Pixel::Three,
                (false, true) => Pixel::Two,
                (true, false) => Pixel::One,
                (false, false) => Pixel::Zero,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }

    fn read_tile_set_data_as_byte(&self, index: usize) -> u8 {
        self.raw_tile_vram[index]
    }

    fn write_oam_data(&mut self, index: usize, byte: u8) {
        self.raw_oam[index] = byte;
        let byte_pos = index & 0x3;
        let oam_index = (index & 0xFFFC) / 4;

        //println!("{:#X} -> {:#X}, {:#X}", index, oam_index, byte_pos);
        self.oam[oam_index][byte_pos] = byte;
    }

    fn can_access_memory(&mut self, mode: Mode) {
        match mode {
            Mode::HBlank => todo!(),  // Can only access: VRAM, OAM, CGB palletes
            Mode::VBlank => todo!(),  // Can only access: VRAM, OAM, CGB palletes
            Mode::OAM => todo!(),     // Can only access: VRAM, CGB palletes
            Mode::Drawing => todo!(), // Can only access: None!
        }
    }

    //
    //  Display
    //

    pub fn enable_lcd(&mut self) {
        self.lcdc |= 0b1010000; //bus.ram.ram[0xFF40] = 0b1010000 | self.vram[0xFF40];
    }

    pub fn is_lcd_enabled(&self) -> bool {
        !(self.lcdc & 0b1000_0000 == 0)
    }

    fn get_address_mode(&self) -> u16 {
        match self.lcdc & 0x10 {
            0 => 0x8800,
            _ => 0x8000,
        }
    }

    fn get_sprite_height(&self) -> u8 {
        match self.lcdc & 0b100 {
            0 => 8,
            _ => 16,
        }
    }

    pub fn get_display(&self) -> &[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3] {
        &self.screen_buffer
    }

    pub fn get_debug_display(&self, buffer: &mut [[Pixel; 16 * 8]; 32 * 8]) {
        let mut tile_no = 0;

        for tile_y in 0..24 {
            for tile_x in 0..16 {
                let start_y = tile_y * 8;
                let start_x = tile_x * 8;

                let tile: Tile = self.tile_set[tile_no];

                for (dy, row) in tile.iter().enumerate() {
                    for (dx, pixel) in row.iter().enumerate() {
                        buffer[start_y + dy][start_x + dx] = *pixel;
                    }
                }

                tile_no += 1;
            }
        }
    }

    pub fn load_tile_data(&self, tile_no: usize, buffer: &mut Tile) {
        let tile: Tile = self.tile_set[tile_no];
        for (dy, row) in tile.iter().enumerate() {
            for (dx, pixel) in row.iter().enumerate() {
                buffer[dy][dx] = *pixel;
            }
        }
    }

    fn copy_from_tile_set(&self, tile_id: usize) -> Tile {
        match self.get_address_mode() {
            0x8000 => self.tile_set[tile_id as usize],
            _ => self.tile_set[(tile_id as i8 as i16 + 128) as usize],
        }
    }

    fn decode_pixels_from_word(&self, byte1: u8, byte2: u8, buffer: &mut [Pixel; 8]) {
        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;
            let value = match (lsb != 0, msb != 0) {
                (true, true) => Pixel::Three,
                (false, true) => Pixel::Two,
                (true, false) => Pixel::One,
                (false, false) => Pixel::Zero,
            };

            buffer[pixel_index] = value;
        }
    }

    //
    //  Pixel Scanline
    //

    // Fetch a row (SCREEN_WDITH) of pixels at the current LY
    // Does NOT take into account the scroll registers (SCX, SCY) as well
    fn scanline_background(&self, row_buffer: &mut [Pixel; 32 * 8]) {
        // Select appropriate background tile map
        let bg_map: &[u8; 0x3FF + 1] = match self.lcdc & 0x8 {
            0 => &self.tile_map_1,
            _ => &self.tile_map_2,
        };

        // Retrieve row of tiles
        let bg_tm_y = (self.scy as usize + self.ly as usize) / 8; // Background Tile Map Y

        // We are returning the entire row from the background map (32 tiles worth of pixels)
        for tile in 0..32 {
            // Loop through entire row
            // Find x position in that row to retrieve (account for the viewport wrapping around the screen) the tile ID
            let tile_index = bg_map[(bg_tm_y * 32 + tile) % (32 * 32)];

            // Access data of that tile ID (depends on current address mode)
            let tile_data = match self.get_address_mode() {
                0x8000 => self.tile_set[tile_index as usize],
                _ => self.tile_set[(tile_index as i8 as i16 + 128) as usize],
            };

            // A tile is an 8x8 grid of pixels
            // Find which row of the tile we want to display
            let row_index: usize = (self.scy as usize + self.ly as usize) % 8;
            let tile_row_data = tile_data[row_index];

            // Load that row into the buffer
            for dx in 0..8 {
                row_buffer[(tile as usize) * 8 + dx] = tile_row_data[dx];
            }
        }
    }

    // Fetch a row (SCREEN_WDITH) of pixels at the current LY
    fn scanline_sprites(&self, row_buffer: &mut [Option<Pixel>; SCREEN_WIDTH]) {
        // Clear row buffer - ensuring we can detect empty tiles properly
        *row_buffer = [None; SCREEN_WIDTH];

        // Check all cached sprites (max 10) that occur on this scanline (self.ly)
        // The cache is already sored by sprites' x-positions, so we can draw and overwrite sprites in the REVERSE order of this vector
        for sprite in self.scanline_sprite_cache.iter().rev() {
            // Sprite data
            let y_position = sprite[0] as usize;
            let x_position = sprite[1] as usize;
            let tile_index = sprite[2] as usize;
            let attributes = sprite[3];

            let tile_data: Tile = self.copy_from_tile_set(tile_index);
            let row_data: [Pixel; 8] = tile_data[(y_position % 8) as usize];
            // Attempt to draw to row_buffer
            for dx in 0..8 {
                row_buffer[x_position + dx] = Some(row_data[dx]);
            }
        }
    }

    fn scanline_render(&mut self) {
        // Scanline background
        let mut bg_buffer: [Pixel; 32 * 8] = [Pixel::Zero; 32 * 8];
        self.scanline_background(&mut bg_buffer);
        // Scanline sprites
        let mut sprite_buffer: [Option<Pixel>; SCREEN_WIDTH] = [None; SCREEN_WIDTH];
        self.scanline_sprites(&mut sprite_buffer);
        // Scanline window

        // Rotate buffer to simulate starting at SCX
        bg_buffer.rotate_left(self.scx as usize);
        // Trim to screen-width (since scanline_background returns 32 tiles of pixels from the background, and the viewport only supports 20 tiles of pixels in a row)
        let background = &bg_buffer[0..SCREEN_WIDTH];

        // Merge scanlines into final result to be displayed
        for (index, bg_pixel) in background.iter().enumerate() {
            let (mut r, mut g, mut b) = self.decode_bg_pixel(*bg_pixel);

            /*
            let sprite_pixel = sprite_buffer[index];
            if let Some(pixel) = sprite_pixel {
                (r, g, b) = self.decode_pixel_color(pixel, self.ob_palette_1, true);
            }
            */

            self.screen_buffer[(self.ly as usize * SCREEN_WIDTH * 3) + (index * 3) + 0] = r;
            self.screen_buffer[(self.ly as usize * SCREEN_WIDTH * 3) + (index * 3) + 1] = g;
            self.screen_buffer[(self.ly as usize * SCREEN_WIDTH * 3) + (index * 3) + 2] = b;
        }
    }

    fn build_sprite_cache(&mut self) {
        self.scanline_sprite_cache = Vec::with_capacity(10);
        for sprite in self.oam {
            let y_position = sprite[0];
            let x_position = sprite[1];

            if x_position > 0
                && self.ly + 16 >= y_position
                && self.ly + 16 < y_position + self.get_sprite_height()
                && self.scanline_sprite_cache.len() < 10
            {
                self.scanline_sprite_cache.push(sprite);
            }
        }

        // Sort by x-position
        self.scanline_sprite_cache.sort_by(|a, b| a[1].cmp(&b[1]));
    }

    pub fn tick(&mut self, _cycles: u16) -> Option<Interrupt> {
        let mut raised_interrupt: Option<Interrupt> = None;

        if !self.is_lcd_enabled() {
            //println!("LCD DISABLED!");
            return None;
        }

        self.mode_cycles += _cycles;
        let mut cycles_left = _cycles;

        while cycles_left > 0 {
            // Expend them
            let ticks_consumed = if cycles_left > 80 { 80 } else { cycles_left }; // Consume at most 80 dots per cycle
            self.mode_cycles += ticks_consumed;
            cycles_left -= ticks_consumed;

            // Process
            //println!("consuming: {}", ticks_consumed);

            if self.ly < 144 {
                // Normal
                if self.mode_cycles < 80 {
                    //println!("\tOAM");
                    // OAM
                    self.build_sprite_cache();
                    raised_interrupt = raised_interrupt.or(self.set_mode(Mode::OAM));
                } else if self.mode_cycles < (80 + 172) {
                    //println!("\tline render");
                    //println!("bg: {:#08b}", self.bg_palette);
                    self.scanline_render();
                    raised_interrupt = raised_interrupt.or(self.set_mode(Mode::Drawing));
                } else if self.mode_cycles < 456 {
                    //println!("\thblank");
                    raised_interrupt = raised_interrupt.or(self.set_mode(Mode::HBlank));
                } else {
                    //println!("\t=> next line!");
                    self.ly = (self.ly + 1) % 154;
                    raised_interrupt = raised_interrupt.or(self.set_mode(Mode::OAM));
                    raised_interrupt = raised_interrupt.or(self.check_lyc_interrupt());
                    self.mode_cycles = self.mode_cycles % 456;
                }
            } else {
                //println!("\tVBLANK");
                // VBlank Lines
                self.set_mode(Mode::VBlank);
                if self.mode_cycles > 456 {
                    self.ly = (self.ly + 1) % 154;

                    raised_interrupt = raised_interrupt.or(self.check_lyc_interrupt());

                    if self.ly < 144 {
                        self.set_mode(Mode::OAM);
                    }

                    self.mode_cycles = self.mode_cycles % 456;
                }
            }
        }

        // Return possible (STAT/LCD) interrupt
        raised_interrupt
    }

    fn check_lyc_interrupt(&mut self) -> Option<Interrupt> {
        if self.ly == self.lyc && self.stat & 0b10_0000 != 0 {
            self.stat |= 0b10;
            Some(Interrupt::LCD)
        } else {
            self.stat &= !0b10;
            None
        }
    }

    // Note: not currently being utilized
    fn decode_bg_pixel(&self, pixel: Pixel) -> (u8, u8, u8) {
        self.decode_pixel_color(pixel, self.bg_palette, false)
    }

    // Note: not currently being utilized
    fn decode_pixel_color(&self, pixel: Pixel, pallete: u8, transparent: bool) -> (u8, u8, u8) {
        let bits = match pixel {
            Pixel::Three => (pallete >> 6) & 0x3,
            Pixel::Two => (pallete >> 4) & 0x3,
            Pixel::One => (pallete >> 2) & 0x3,
            Pixel::Zero => (pallete >> 0) & 0x3,
        };
        let color = Color::from(Pixel::from(bits as u8));

        (color.r, color.g, color.b)
    }
}

fn log(s: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("gb-vram-log")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", s) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn log_vec(vec: Vec<u8>) {
    log(format!("{:?}", vec));
    log(format!("\n"));
}

pub fn log_data(tile_set: [Tile; 384]) {
    for tile in tile_set.iter() {
        print!("{:?}\n", tile);
    }
    println!("--------------------------------\n");
}

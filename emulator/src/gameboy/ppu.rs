use sdl2::pixels::Color;
use std::{fmt, fs::OpenOptions, io::prelude::*};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::Bus;

const VRAM_SIZE: usize = (0x9FFF - 0x8000) + (0xFE9F - 0xFE00);

const TILE_MAP_SIZE: usize = 0x3FF;

pub struct PPU {
    // Memory Map (entirety of VRAM) 0x8000-0x9FFF and 0xFE00-0xFE9F
    raw_tile_vram: [u8; 384 * 16],
    pub tile_set: [Tile; 384],   // Tile Set Blocks 0-3 - 0x8000-0x97FF
    tile_map_1: [u8; 0x3FF + 1], // Background Map 1 - 0x9800 - 0x9BFF    // Each entry (byte, u8) is a tile number (tile located in tile_set)
    tile_map_2: [u8; 0x3FF + 1], // Background Map 2 - 0x9C00 - 0x9FFF    // "                                                                "
    raw_oam: [u8; 0xA0], // Object Attribute Memory - 0xFE00 - 0xFE9F // Each entry is 4 bytes, [u8; 4] - https://gbdev.io/pandocs/OAM.html#object-attribute-memory-oam
    oam: [[u8; 4]; 40],
    // IO Registers 0xFF40-0xFF4B
    lcdc: u8,         // PPU control register - 0xFF40
    stat: u8,         // PPU status register - 0xFF41
    scy: u8,          // Vertical scroll register - 0xFF42
    scx: u8,          // Horizontal scroll register - 0xFF43
    pub ly: u8,       // Scanline register - 0xFF44
    lyc: u8,          // LY Compare - 0xFF45
    bg_palette: u8,   // Background color palette - 0xFF47
    ob_palette_1: u8, // Object color palette 1 - 0xFF48
    ob_palette_2: u8, // Object color palette 2 - 0xFF49
    wy: u8,           // Window Y position - 0xFF4A
    wx: u8,           // Window X position - 0xFF4B
    // Internal data structures
    screen_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    background_fifo: [Option<Pixel>; 16],
    sprite_fifo: [Option<u8>; 16], // Sprite indexes
    mode: Mode,
    oam_index: usize,
    current_scanline_cycles: f64,
    pixel_fetcher: PixelFetcher,
}

// https://github.com/Hacktix/GBEDG/blob/master/ppu/index.md#the-concept-of-ppu-modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    HBlank = 0, // Takes place after the current scanline has completed, pads the duration of the line scan to 456 T-Cycles - effectively a pause for the PPU.
    VBlank = 1, // The psuedo lines (144-153), taking place after the entire screen has been scanned
    OAM = 2, // Entered at the start of every scanline (except for V-Blank), before pixels are actually drawn to the screen. Renders sprites.
    Drawing = 3, // Where the PPU transfers pixels to the LCD.
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum FetchState {
    ReadTile,
    ReadData0,
    ReadData1,
}

struct PixelFetcher {
    state: FetchState,
    tile_index: Option<u8>,
    upper_nibble: Option<u8>,
    lower_nibble: Option<u8>,
    scanline_x: u8,
    scanline_y: u8,
}

impl PixelFetcher {
    pub fn new() -> Self {
        PixelFetcher {
            state: FetchState::ReadTile,
            tile_index: None,
            upper_nibble: None,
            lower_nibble: None,
            scanline_x: 0,
            scanline_y: 0,
        }
    }

    pub fn next(&mut self) {
        self.state = match self.state {
            FetchState::ReadTile => FetchState::ReadData0,
            FetchState::ReadData0 => FetchState::ReadData1,
            FetchState::ReadData1 => {
                self.tile_index = None;
                self.upper_nibble = None;
                self.lower_nibble = None;

                FetchState::ReadTile
            }
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
            Pixel::Three => Color::RGB(255, 255, 255),
            Pixel::One => Color::RGB(255, 0, 0),
            Pixel::Two => Color::RGB(0, 255, 0),
            Pixel::Zero => Color::RGB(0, 0, 0),
        }
    }
}
impl std::convert::From<Pixel> for u8 {
    fn from(pixel: Pixel) -> u8 {
        match pixel {
            Pixel::Three => 0b11,
            Pixel::One => 0b10,
            Pixel::Two => 0b01,
            Pixel::Zero => 0b00,
        }
    }
}
impl std::convert::From<u8> for Pixel {
    fn from(byte: u8) -> Pixel {
        match byte {
            0b11 => Pixel::Three,
            0b10 => Pixel::One,
            0b01 => Pixel::Two,
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
            oam: [[0; 4]; 40],
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
            screen_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            background_fifo: [None; 16],
            sprite_fifo: [None; 16],
            mode: Mode::OAM,
            oam_index: 0,
            current_scanline_cycles: 0f64,
            pixel_fetcher: PixelFetcher::new(),
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
        let oam_index = index & 0xFFFC;

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

    pub fn enable_lcd(&mut self, bus: &mut Bus) {
        self.lcdc |= 0b1010000; //bus.ram.ram[0xFF40] = 0b1010000 | self.vram[0xFF40];
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

    fn get_background_map(&self, buffer: &mut [Tile; 360]) {
        let bg_map: &[u8; 0x3FF + 1] = match self.lcdc & 0x8 {
            0 => &self.tile_map_1,
            _ => &self.tile_map_2,
        };

        *buffer = [empty_tile(); 20 * 18];
        // Background consists of 32x32 tiles, or 256x256 pixels
        // The viewport (Game Boy screen) can only show 20x18 tiles (160x144 pixels)
        // let start_index = (self.scx + self.scy * 20) as usize;
        let start_index = (((self.scx as u16) / 8) + ((self.scy as u16) / 8) * 20) as usize;
        let end_index: usize = start_index + (20 * 18);

        for (i, tile_id) in bg_map[start_index..end_index].iter().enumerate() {
            buffer[i] = match self.get_address_mode() {
                0x8000 => self.tile_set[*tile_id as usize],
                _ => self.tile_set[(*tile_id as i8 as i16 + 128) as usize],
            };
        }
    }

    pub fn get_display(&self, bus: &Bus, scx: u8, scy: u8, buffer: &mut [[Pixel; 20 * 8]; 18 * 8]) {
        let mut bg_tile_buffer: [Tile; 360] = [empty_tile(); 360];
        self.get_background_map(&mut bg_tile_buffer);

        // 20x18 view of the background
        // Begin at (scx, scy)

        /*
        for (i, tile_data) in bg_tile_buffer.iter().enumerate() {
            let start_row = (i / 20) * 8;
            let start_col = (i * 8) % (20 * 8);
            //println!("{}, {}", start_row, start_col);
            for dy in 0..8 {
                for dx in 0..8 {
                    buffer[start_row + dy][start_col + dx] = tile_data[dy][dx];
                }
            }
        }*/

        let mut current_ly = self.ly;
        println!("c ly: {:?}", current_ly);
        for row in 0..SCREEN_HEIGHT {
            buffer[row] = self.scanline_render(current_ly);
            current_ly = (current_ly.wrapping_add(1) % SCREEN_HEIGHT as u8);
        }
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

    fn copy_from_tile_set(&self, tile_id: u8) -> Tile {
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
    //  Pixel FIFO
    //

    fn scanline_render(&self, ly: u8) -> [Pixel; SCREEN_WIDTH] {
        let mut bg_tile_buffer: [Tile; 360] = [empty_tile(); 360];
        self.get_background_map(&mut bg_tile_buffer);

        let mut buffer: [Pixel; SCREEN_WIDTH] = [Pixel::Three; SCREEN_WIDTH];

        // Position in screen/display
        let mut screen_y = (self.scy as u16 + ly as u16) % SCREEN_HEIGHT as u16;
        let mut screen_x = self.scx as u16;

        // Convert to tilemap indexes
        let tile_map_y = (screen_y / 8) as usize;
        let tile_row = (screen_y - ((screen_y / 8) * 8)) as usize;
        let initial_tile_map_x = screen_x / 8;

        for tile in 0..(SCREEN_WIDTH / 8 as usize) {
            // ...
            let tile_map_x =
                ((initial_tile_map_x + tile as u16) % (SCREEN_WIDTH / 8) as u16) as usize;
            let inline_index = (tile_map_y * (SCREEN_WIDTH / 8)) + tile_map_x;
            let tile_data = bg_tile_buffer[inline_index];
            let row_data = tile_data[tile_row];

            for dx in 0..8 {
                buffer[(tile * 8) + dx] = row_data[dx];
            }
        }

        buffer
    }

    fn fetch_background_fifo(&mut self) -> f64 {
        if PPU::queue_size(&self.background_fifo).unwrap_or(9) < 8 {
            // Load another 8 bytes
            match self.pixel_fetcher.state {
                FetchState::ReadTile => {}
                FetchState::ReadData0 => {}
                FetchState::ReadData1 => {}
            };
            self.pixel_fetcher.next();
            return 2f64;
        }
        return 2f64;
    }
    fn fetch_sprite_fifo(&mut self) {}
    fn fetch_window_fifo(&mut self) {}

    fn single_oam_scan(&mut self) -> f64 {
        // Search for sprites that may be needed for the current scanline (LY)
        // Consumes 2 cycles per entry check => 80 cycles total for a full scan
        // This method will consume two cycles, just scanning a single entry
        let sprite_data = self.oam[self.oam_index];

        let y_position: u8 = sprite_data[0];
        let x_position: u8 = sprite_data[1];
        let tile_index: u8 = sprite_data[2]; // Always uses $8000 addressing (its an unsigned 8 bit integer)
        let sprite_flags: u8 = sprite_data[3];

        // Conditions to add to buffer
        let sprite_height = self.get_sprite_height();
        let sprites_in_buffer = PPU::queue_size(&self.sprite_fifo);

        if x_position > 0
            && self.ly + 16 >= y_position
            && self.ly + 16 < y_position + sprite_height
            && sprites_in_buffer.unwrap_or_else(|| 11) < 10
        {
            // add to sprite queue
            PPU::push_sprite_into_buffer(&mut self.sprite_fifo, self.oam_index as u8);
        }

        self.oam_index += 1;

        return 2f64;
    }

    // Some if non-full, None if full
    fn queue_size<T, const N: usize>(buffer: &[Option<T>; N]) -> Option<usize> {
        for i in 0..N {
            if buffer[i].is_some() {
                return Some(i + 1);
            }
        }
        return None;
    }

    fn push_pixels_into_buffer(buffer: &mut [Option<Pixel>; 16], pixels: &[Pixel; 8]) {
        let mut pushed = 0;
        let mut index = 0;
        while pushed < 8 {
            if buffer[index].is_none() {
                buffer[index] = Some(pixels[pushed]);
                pushed += 1;
            }

            index += 1;
        }

        if pushed > 0 && pushed < 8 {
            panic!("Error pushing pixels into buffer, already full?!?");
        }
    }

    fn push_sprite_into_buffer(buffer: &mut [Option<u8>; 16], sprite_id: u8) -> bool {
        for index in 0..16 {
            if buffer[index].is_none() {
                buffer[index] = Some(sprite_id);
                return true;
            }
        }

        return false;
    }

    pub fn tick(&mut self, _cycles: u8) {
        self.ly = (self.ly + 1) % 144;
    }

    fn step_wip(&mut self, t_cycles: f64) {
        let mut cycles_left = t_cycles;

        while cycles_left > 0.0 {
            let cycles_consumed = match self.mode {
                Mode::HBlank => 1f64, // Effectively just padding
                Mode::VBlank => todo!(),
                Mode::OAM => self.single_oam_scan(), // Entered at the start of every scanline (except for V-Blank)
                Mode::Drawing => self.fetch_background_fifo(),
            };

            cycles_left -= cycles_consumed;
            self.current_scanline_cycles += cycles_consumed;

            // TODO: reset x position as well?
            match self.mode {
                Mode::HBlank => {
                    if self.current_scanline_cycles >= 456f64 {
                        self.mode = Mode::OAM;
                        self.ly += 1;
                        if self.ly > 144 {
                            self.mode = Mode::VBlank;
                        }
                    }
                }
                Mode::VBlank => {
                    self.ly += 1;
                    if self.ly >= 154 {
                        self.mode = Mode::OAM;
                        self.ly = 0;
                    }
                }
                Mode::OAM => {
                    if self.oam_index >= 40 {
                        self.mode = Mode::Drawing;
                        self.oam_index = 0;
                    }
                }
                Mode::Drawing => todo!(),
            }
        }
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

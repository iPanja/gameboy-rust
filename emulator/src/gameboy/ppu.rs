pub struct PPU {}

/* Registers
    0xFF40 - LCDC - PPU control register
    0xFF41 - STAT - PPU status register
    0xFF42 - SCY - Vertical scroll register
    0xFF43 - SCX - Horizontal scroll register
    0xFF44 - LY - Scanline register
*/

/* VRAM Tile Data
    Block 0 - $8000-87FF
    Block 1 - $8800-8FFF
    Block 2 - $9000-97FF
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
    For the Background or Window, the color ID => a pallete
    For Objects, color ID 0 = transparent, 1-3 => a pallete

    Each tile occupies 16 bytes, where each line is represented by 2 bytes.
    For each line:
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
    > Taken from tile blocks 0 and 1, locatd at $8000-8FFF and have unsigned numbering
    > Object attributes are stored in the OAM at $FE00-FE9F
        > Byte 0 - Y Position
        > Byte 1 - X Position
        > Byte 2 - Tile Index
        > Byte 3 - Attributes/Flags
*/
use std::{fmt::Write, result};

pub(crate) fn parse_opcode(
    opcode: u8,
    next_byte: u8,
    next_word: u16,
    w: &mut impl Write,
) -> Result<(u8, u8), String> {
    let mut r: &str;
    // Vec<Box<dyn FnOnce(u16) -> String>>
    let (result, cycles, bytes) = match opcode {
        // FORMAT 0x00 => { statement ; clock_cycles }
        0xCB => {
            return parse_opcode_cb(opcode, next_byte, next_word, w);
        }
        //
        // 8-Bit Loads
        //
        // LD nn, n
        // A
        0x06 => (write!(w, "LD B, {}", next_byte), 8, 2),
        0x0E => (write!(w, "LD C, {}", next_byte), 8, 2),
        0x16 => (write!(w, "LD D, {}", next_byte), 8, 2),
        0x1E => (write!(w, "LD D, {}", next_byte), 8, 2),
        0x26 => (write!(w, "LD D, {}", next_byte), 8, 2),
        0x2E => (write!(w, "LD D, {}", next_byte), 8, 2),
        // LD r1, r2
        0x7F => (write!(w, "LD A, A",), 4, 1),
        0x78 => (write!(w, "LD A, B",), 4, 1),
        0x79 => (write!(w, "LD A, C",), 4, 1),
        0x7A => (write!(w, "LD A, D",), 4, 1),
        0x7B => (write!(w, "LD A, E",), 4, 1),
        0x7C => (write!(w, "LD A, H",), 4, 1),
        0x7D => (write!(w, "LD A, L",), 4, 1),
        0x7E => (write!(w, "LD A, (HL)",), 8, 1),
        // B
        0x40 => (write!(w, "LD B, B",), 4, 1),
        0x41 => (write!(w, "LD B, C",), 4, 1),
        0x42 => (write!(w, "LD B, D",), 4, 1),
        0x43 => (write!(w, "LD B, E",), 4, 1),
        0x44 => (write!(w, "LD B, H",), 4, 1),
        0x45 => (write!(w, "LD B, L",), 4, 1),
        0x46 => (write!(w, "LD B, (HL)",), 8, 1),
        // C
        0x48 => (write!(w, "LD C, B",), 4, 1),
        0x49 => (write!(w, "LD C, C",), 4, 1),
        0x4A => (write!(w, "LD C, D",), 4, 1),
        0x4B => (write!(w, "LD C, E",), 4, 1),
        0x4C => (write!(w, "LD C, H",), 4, 1),
        0x4D => (write!(w, "LD C, L",), 4, 1),
        0x4E => (write!(w, "LD C, (HL)",), 8, 1),
        // D
        0x50 => (write!(w, "LD D, B",), 4, 1),
        0x51 => (write!(w, "LD D, C",), 4, 1),
        0x52 => (write!(w, "LD D, D",), 4, 1),
        0x53 => (write!(w, "LD D, E",), 4, 1),
        0x54 => (write!(w, "LD D, H",), 4, 1),
        0x55 => (write!(w, "LD D, L",), 4, 1),
        0x56 => (write!(w, "LD D, (HL)",), 8, 1),
        // E
        0x58 => (write!(w, "LD E, B",), 4, 1),
        0x59 => (write!(w, "LD E, C",), 4, 1),
        0x5A => (write!(w, "LD E, D",), 4, 1),
        0x5B => (write!(w, "LD E, E",), 4, 1),
        0x5C => (write!(w, "LD E, H",), 4, 1),
        0x5D => (write!(w, "LD E, L",), 4, 1),
        0x5E => (write!(w, "LD E, (HL)",), 8, 1),
        // H
        0x60 => (write!(w, "LD H, B",), 4, 1),
        0x61 => (write!(w, "LD H, C",), 4, 1),
        0x62 => (write!(w, "LD H, D",), 4, 1),
        0x63 => (write!(w, "LD H, E",), 4, 1),
        0x64 => (write!(w, "LD H, H",), 4, 1),
        0x65 => (write!(w, "LD H, L",), 4, 1),
        0x66 => (write!(w, "LD H, (HL)",), 8, 1),
        // L
        0x68 => (write!(w, "LD L, B",), 4, 1),
        0x69 => (write!(w, "LD L, C",), 4, 1),
        0x6A => (write!(w, "LD L, D",), 4, 1),
        0x6B => (write!(w, "LD L, E",), 4, 1),
        0x6C => (write!(w, "LD L, H",), 4, 1),
        0x6D => (write!(w, "LD L, L",), 4, 1),
        0x6E => (write!(w, "LD L, (HL)",), 8, 1),
        // (HL)
        0x70 => (write!(w, "LD (HL), B",), 8, 1),
        0x71 => (write!(w, "LD (HL), C",), 8, 1),
        0x72 => (write!(w, "LD (HL), D",), 8, 1),
        0x73 => (write!(w, "LD (HL), E",), 8, 1),
        0x74 => (write!(w, "LD (HL), H",), 8, 1),
        0x75 => (write!(w, "LD (HL), L",), 8, 1),
        0x36 => (write!(w, "LD (HL), {}", next_byte), 12, 2),
        // LD A, n
        0x78 => (write!(w, "LD A, B",), 4, 1),
        0x7F => (write!(w, "LD A, A",), 4, 1),
        0x79 => (write!(w, "LD A, C",), 4, 1),
        0x7A => (write!(w, "LD A, D",), 4, 1),
        0x7B => (write!(w, "LD A, E",), 4, 1),
        0x7C => (write!(w, "LD A, H",), 4, 1),
        0x7D => (write!(w, "LD A, L",), 4, 1),
        0x0A => (write!(w, "LD A, (BC)",), 4, 1),
        0x1A => (write!(w, "LD A, (DE)",), 4, 1),
        0x7E => (write!(w, "LD A, (HL)",), 4, 1),
        0xFA => (write!(w, "LD A, ({:#X})", next_word), 4, 3),
        0x3E => (write!(w, "LD A, {:#X}", next_byte), 4, 2),
        // LD n, A
        0x7F => (write!(w, "LD A, A",), 4, 1),
        0x47 => (write!(w, "LD B, A",), 4, 1),
        0x4F => (write!(w, "LD C, A",), 4, 1),
        0x57 => (write!(w, "LD D, A",), 4, 1),
        0x5F => (write!(w, "LD E, A",), 4, 1),
        0x67 => (write!(w, "LD H, A",), 4, 1),
        0x6F => (write!(w, "LD L, A",), 4, 1),
        0x02 => (write!(w, "LD (BC), A",), 8, 1),
        0x12 => (write!(w, "LD (DE), A",), 8, 1),
        0x77 => (write!(w, "LD (HL), A",), 8, 1),
        0xEA => (write!(w, "LD ({:#X}), A", next_word), 16, 3),
        // LD A, (C)
        0xF2 => (write!(w, "LD A, ($FF00 + C)",), 8, 1),
        // LD (C), A
        0xE2 => (write!(w, "LD ($FF00 + C), A",), 8, 1),
        // LD A, (HLD) ; LD A, (HL-) ; LDD A, (HL) - 0x3A
        0x3A => (write!(w, "LDD A, (HL-)",), 8, 1),
        // LD (HLD), A ; LD (HL-), A ; LDD (HL), A - 0x32
        0x32 => (write!(w, "LDD (HL-), A",), 8, 1),
        // LD A, (HLI) ; LD A, (HL+) ; LDI A, (HL) - 0x2A
        0x2A => (write!(w, "LDD A, (HL+)",), 8, 1),
        // LD (HLI), A ; LD (HL+), A ; LDI (HL), A - 0x22
        0x22 => (write!(w, "LDD (HL+), A",), 8, 1),
        // LDH (n), A - 0xE0
        0xE0 => (write!(w, "LDH ($FF00 + {:#X}), A", next_byte), 12, 2),
        // LDH A, (n)
        0xF0 => (write!(w, "LDH A, ($FF00 + {:#X})", next_byte), 12, 2),
        //
        // 16-Bit Loads
        //
        // LD n, nn
        0x01 => (write!(w, "LD BC, {:#X}", next_word), 12, 3),
        0x11 => (write!(w, "LD DE, {:#X}", next_word), 12, 3),
        0x21 => (write!(w, "LD HL, {:#X}", next_word), 12, 3),
        0x31 => (write!(w, "LD SP, {:#X}", next_word), 12, 3),
        // LD SP, HL
        0xF9 => (write!(w, "LD SP, HL",), 8, 1),
        // LD HL, SP+n
        // LDHL SP, n
        0xF8 => (write!(w, "LDHL SP, {:#X}", next_byte), 12, 2),
        // LD (nn), SP - 0x08
        0x08 => (write!(w, "LD ({:#X}), SP", next_word), 20, 3),
        // PUSH nn
        0xF5 => (write!(w, "PUSH AF",), 16, 1),
        0xC5 => (write!(w, "PUSH BC",), 16, 1),
        0xD5 => (write!(w, "PUSH DE",), 16, 1),
        0xE5 => (write!(w, "PUSH HL",), 16, 1),
        // POP nn
        0xF1 => (write!(w, "POP AF",), 12, 1),
        0xC1 => (write!(w, "POP BC",), 12, 1),
        0xD1 => (write!(w, "POP DE",), 12, 1),
        0xE1 => (write!(w, "POP HL",), 12, 1),
        //
        //  8-Bit ALU
        //
        // ADD A, n
        0x87 => (write!(w, "ADD A, A",), 4, 1),
        0x80 => (write!(w, "ADD A, B",), 4, 1),
        0x81 => (write!(w, "ADD A, C",), 4, 1),
        0x82 => (write!(w, "ADD A, D",), 4, 1),
        0x83 => (write!(w, "ADD A, E",), 4, 1),
        0x84 => (write!(w, "ADD A, H",), 4, 1),
        0x85 => (write!(w, "ADD A, L",), 4, 1),
        0x86 => (write!(w, "ADD A, (HL)",), 8, 1),
        0xC6 => (write!(w, "ADD A, {:#X}", next_byte), 8, 2),
        // ADC A, n
        0x8F => (write!(w, "ADC A, A",), 4, 1),
        0x88 => (write!(w, "ADC A, B",), 4, 1),
        0x89 => (write!(w, "ADC A, C",), 4, 1),
        0x8A => (write!(w, "ADC A, D",), 4, 1),
        0x8B => (write!(w, "ADC A, E",), 4, 1),
        0x8C => (write!(w, "ADC A, H",), 4, 1),
        0x8D => (write!(w, "ADC A, L",), 4, 1),
        0x8E => (write!(w, "ADC A, (HL)",), 8, 1),
        0xCE => (write!(w, "ADC A, {:#X}", next_byte), 8, 2),
        // SUB n
        0x97 => (write!(w, "SUB A",), 4, 1),
        0x90 => (write!(w, "SUB B",), 4, 1),
        0x91 => (write!(w, "SUB C",), 4, 1),
        0x92 => (write!(w, "SUB D",), 4, 1),
        0x93 => (write!(w, "SUB E",), 4, 1),
        0x94 => (write!(w, "SUB H",), 4, 1),
        0x95 => (write!(w, "SUB L",), 4, 1),
        0x96 => (write!(w, "SUB (HL)",), 8, 1),
        0xD6 => (write!(w, "SUB {:#X}", next_byte), 8, 2),
        // SBC A, n
        0x9F => (write!(w, "SBC A, A",), 4, 1),
        0x98 => (write!(w, "SBC A, B",), 4, 1),
        0x99 => (write!(w, "SBC A, C",), 4, 1),
        0x9A => (write!(w, "SBC A, D",), 4, 1),
        0x9B => (write!(w, "SBC A, E",), 4, 1),
        0x9C => (write!(w, "SBC A, H",), 4, 1),
        0x9D => (write!(w, "SBC A, L",), 4, 1),
        0x9E => (write!(w, "SBC A, (HL)",), 8, 1),
        0xDE => (write!(w, "SBC A, {:#X}", next_byte), 8, 2),
        // AND n
        0xA7 => (write!(w, "AND A",), 4, 1),
        0xA0 => (write!(w, "AND B",), 4, 1),
        0xA1 => (write!(w, "AND C",), 4, 1),
        0xA2 => (write!(w, "AND D",), 4, 1),
        0xA3 => (write!(w, "AND E",), 4, 1),
        0xA4 => (write!(w, "AND H",), 4, 1),
        0xA5 => (write!(w, "AND L",), 4, 1),
        0xA6 => (write!(w, "AND (HL)",), 8, 1),
        0xE6 => (write!(w, "AND {:#X}", next_byte), 8, 2),
        // OR n
        0xB7 => (write!(w, "OR A",), 4, 1),
        0xB0 => (write!(w, "OR B",), 4, 1),
        0xB1 => (write!(w, "OR C",), 4, 1),
        0xB2 => (write!(w, "OR D",), 4, 1),
        0xB3 => (write!(w, "OR E",), 4, 1),
        0xB4 => (write!(w, "OR H",), 4, 1),
        0xB5 => (write!(w, "OR L",), 4, 1),
        0xB6 => (write!(w, "OR (HL)",), 8, 1),
        0xF6 => (write!(w, "OR {:#X}", next_byte), 8, 2),
        // XOR n
        0xAF => (write!(w, "XOR A",), 4, 1),
        0xA8 => (write!(w, "XOR B",), 4, 1),
        0xA9 => (write!(w, "XOR C",), 4, 1),
        0xAA => (write!(w, "XOR D",), 4, 1),
        0xAB => (write!(w, "XOR E",), 4, 1),
        0xAC => (write!(w, "XOR H",), 4, 1),
        0xAD => (write!(w, "XOR L",), 4, 1),
        0xAE => (write!(w, "XOR (HL)",), 8, 1),
        0xEE => (write!(w, "XOR {:#X}", next_byte), 8, 2),
        // CP n
        0xBF => (write!(w, "CP A",), 4, 1),
        0xB8 => (write!(w, "CP B",), 4, 1),
        0xB9 => (write!(w, "CP C",), 4, 1),
        0xBA => (write!(w, "CP D",), 4, 1),
        0xBB => (write!(w, "CP E",), 4, 1),
        0xBC => (write!(w, "CP H",), 4, 1),
        0xBD => (write!(w, "CP L",), 4, 1),
        0xBE => (write!(w, "CP (HL)",), 8, 1),
        0xFE => (write!(w, "CP {:#X}", next_byte), 8, 2),
        // INC n
        0x3C => (write!(w, "INC A",), 4, 1),
        0x04 => (write!(w, "INC B",), 4, 1),
        0x0C => (write!(w, "INC C",), 4, 1),
        0x14 => (write!(w, "INC D",), 4, 1),
        0x1C => (write!(w, "INC E",), 4, 1),
        0x24 => (write!(w, "INC H",), 4, 1),
        0x2C => (write!(w, "INC L",), 4, 1),
        0x34 => (write!(w, "INC (HL)",), 12, 1),
        // DEC n
        0x3D => (write!(w, "DEC A",), 4, 1),
        0x05 => (write!(w, "DEC B",), 4, 1),
        0x0D => (write!(w, "DEC C",), 4, 1),
        0x15 => (write!(w, "DEC D",), 4, 1),
        0x1D => (write!(w, "DEC E",), 4, 1),
        0x25 => (write!(w, "DEC H",), 4, 1),
        0x2D => (write!(w, "DEC L",), 4, 1),
        0x35 => (write!(w, "DEC (HL)",), 4, 1),
        //
        // 16-Bit Arithmetic
        //
        // ADD HL, n
        0x09 => (write!(w, "ADD HL, BC",), 8, 1),
        0x19 => (write!(w, "ADD HL, DE",), 8, 1),
        0x29 => (write!(w, "ADD HL, HL",), 8, 1),
        0x39 => (write!(w, "ADD HL, SP",), 8, 1),
        // ADD SP, n
        0xE8 => (write!(w, "ADD HL, BC",), 16, 1), // i8 signed immediate
        // INC nn
        0x03 => (write!(w, "INC BC",), 8, 1),
        0x13 => (write!(w, "INC DE",), 8, 1),
        0x23 => (write!(w, "INC HL",), 8, 1),
        0x33 => (write!(w, "INC SP",), 8, 1),
        // DEC nn
        0x0B => (write!(w, "DEC BC",), 8, 1),
        0x1B => (write!(w, "DEC DE",), 8, 1),
        0x2B => (write!(w, "DEC HL",), 8, 1),
        0x3B => (write!(w, "DEC SP",), 8, 1),
        // DAA
        0x27 => (write!(w, "DAA",), 4, 1),
        // CPL
        0x2F => (write!(w, "CPL",), 4, 1),
        // CCF
        0x3F => (write!(w, "CCF",), 4, 1),
        // SCF
        0x37 => (write!(w, "SCF",), 4, 1),
        // NOP
        0x00 => (write!(w, "NOP",), 4, 1),
        // HALT
        0x76 => (write!(w, "HALT",), 4, 1),
        // STOP
        // DI
        // EI
        //
        // Shifts & Rotations
        //
        // RLCA
        0x07 => (write!(w, "RLCA",), 4, 1),
        // RLA
        0x17 => (write!(w, "RLA",), 4, 1),
        // RRCA
        0x0F => (write!(w, "RRCA",), 4, 1),
        // RRA
        0x1F => (write!(w, "RRA",), 4, 1),
        //
        // Jumps
        //
        // JP nn
        0xC3 => (write!(w, "JP {:#X}", next_word), 12, 3),
        // JP cc, nn
        0xC2 => (write!(w, "JP NZ, n{:#X}n", next_word), 12, 3),
        0xCA => (write!(w, "JP Z, {:#X}", next_word), 12, 3),
        0xD2 => (write!(w, "JP NC, n{:#X}n", next_word), 12, 3),
        0xDA => (write!(w, "JP C, {:#X}", next_word), 12, 3),
        // JP (HL)
        0xE9 => (write!(w, "JP (HL)",), 4, 1),
        // JR n
        0x18 => (write!(w, "JR {:#X}", next_byte as i8), 8, 2),
        // JR cc, n
        0x20 => (write!(w, "JR NZ, {:#X}", next_byte as i8), 8, 2), // signed i8
        0x28 => (write!(w, "JR Z, {:#X}", next_byte as i8), 8, 2),
        0x30 => (write!(w, "JR NC, {:#X}", next_byte as i8), 8, 2),
        0x38 => (write!(w, "JR C, {:#X}", next_byte as i8), 8, 2),
        //
        // Calls
        //
        // CALL nn
        0xCD => (write!(w, "CALL NZ, {:#X}", next_word as i16), 12, 3), // 2 byte i8 signed
        // CALL cc, nn
        0xC4 => (write!(w, "CALL NZ, {:#X}", next_word as i16), 12, 3), // 2 byte i8 signed
        0xCC => (write!(w, "CALL Z, {:#X}", next_word as i16), 12, 3),
        0xD4 => (write!(w, "CALL NC, {:#X}", next_word as i16), 12, 3),
        0xDC => (write!(w, "CALL C, {:#X}", next_word as i16), 12, 3),
        //
        // Restarts
        //
        // RST n
        0xC7 => (write!(w, "RST $00",), 16, 1),
        0xCF => (write!(w, "RST $08",), 16, 1),
        0xD7 => (write!(w, "RST $10",), 16, 1),
        0xDF => (write!(w, "RST $18",), 16, 1),
        0xE7 => (write!(w, "RST $20",), 16, 1),
        0xEF => (write!(w, "RST $28",), 16, 1),
        0xF7 => (write!(w, "RST $30",), 16, 1),
        0xFF => (write!(w, "RST $38",), 16, 1),
        //
        // Returns
        //
        // RET
        0xC9 => (write!(w, "RET",), 8, 1),
        // RET cc
        0xC0 => (write!(w, "RET NZ",), 8, 1),
        0xC8 => (write!(w, "RET Z",), 8, 1),
        0xD0 => (write!(w, "RET NC",), 8, 1),
        0xD8 => (write!(w, "RET C",), 8, 1),
        // RETI
        0xD9 => (write!(w, "RETI",), 8, 1),
        //
        // Miscellaneous
        //
        // DI
        0xF3 => (write!(w, "DI",), 4, 1),
        // EI
        0xFB => (write!(w, "EI",), 4, 1),
        // NOT FOUND!
        _ => (write!(w, "Unsupported instruction",), 0, 1),
    };

    if let Ok(_) = result {
        Ok((cycles, bytes))
    } else {
        Err("Unsupported instruction".to_string())
    }
}

fn parse_opcode_cb(
    opcode: u8,
    next_byte: u8,
    next_word: u16,
    w: &mut impl Write,
) -> Result<(u8, u8), String> {
    let (result, cycles, bytes) = match opcode {
        // MISC
        // SWAP n
        0x37 => (write!(w, "SWAP A",), 8, 2),
        0x30 => (write!(w, "SWAP B",), 8, 2),
        0x31 => (write!(w, "SWAP C",), 8, 2),
        0x32 => (write!(w, "SWAP D",), 8, 2),
        0x33 => (write!(w, "SWAP E",), 8, 2),
        0x34 => (write!(w, "SWAP H",), 8, 2),
        0x35 => (write!(w, "SWAP L",), 8, 2),
        0x36 => (write!(w, "SWAP (HL)",), 16, 2),
        //
        // Shifts & Rotations
        //
        // RLC n
        0x07 => (write!(w, "RLC A",), 8, 2),
        0x00 => (write!(w, "RLC B",), 8, 2),
        0x01 => (write!(w, "RLC C",), 8, 2),
        0x02 => (write!(w, "RLC D",), 8, 2),
        0x03 => (write!(w, "RLC E",), 8, 2),
        0x04 => (write!(w, "RLC H",), 8, 2),
        0x05 => (write!(w, "RLC L",), 8, 2),
        0x06 => (write!(w, "RLC (HL)",), 16, 2),
        // RL n
        0x17 => (write!(w, "RC A",), 8, 2),
        0x10 => (write!(w, "RC B",), 8, 2),
        0x11 => (write!(w, "RC C",), 8, 2),
        0x12 => (write!(w, "RC D",), 8, 2),
        0x13 => (write!(w, "RC E",), 8, 2),
        0x14 => (write!(w, "RC H",), 8, 2),
        0x15 => (write!(w, "RC L",), 8, 2),
        0x16 => (write!(w, "RC (HL)",), 16, 2),
        // RRC n
        0x0F => (write!(w, "RRC A",), 8, 2),
        0x08 => (write!(w, "RRC B",), 8, 2),
        0x09 => (write!(w, "RRC C",), 8, 2),
        0x0A => (write!(w, "RRC D",), 8, 2),
        0x0B => (write!(w, "RRC E",), 8, 2),
        0x0C => (write!(w, "RRC H",), 8, 2),
        0x0D => (write!(w, "RRC L",), 8, 2),
        0x0E => (write!(w, "RRC (HL)",), 16, 2),
        // RR n
        0x1F => (write!(w, "RR A",), 8, 2),
        0x18 => (write!(w, "RR B",), 8, 2),
        0x19 => (write!(w, "RR C",), 8, 2),
        0x1A => (write!(w, "RR D",), 8, 2),
        0x1B => (write!(w, "RR E",), 8, 2),
        0x1C => (write!(w, "RR H",), 8, 2),
        0x1D => (write!(w, "RR L",), 8, 2),
        0x1E => (write!(w, "RR (HL)",), 16, 2),
        // SLA n
        0x27 => (write!(w, "SLA A",), 8, 2),
        0x20 => (write!(w, "SLA B",), 8, 2),
        0x21 => (write!(w, "SLA C",), 8, 2),
        0x22 => (write!(w, "SLA D",), 8, 2),
        0x23 => (write!(w, "SLA E",), 8, 2),
        0x24 => (write!(w, "SLA H",), 8, 2),
        0x25 => (write!(w, "SLA L",), 8, 2),
        0x26 => (write!(w, "SLA (HL)",), 16, 2),
        // SRA n
        0x2F => (write!(w, "SLA A",), 8, 2),
        0x28 => (write!(w, "SLA B",), 8, 2),
        0x29 => (write!(w, "SLA C",), 8, 2),
        0x2A => (write!(w, "SLA D",), 8, 2),
        0x2B => (write!(w, "SLA E",), 8, 2),
        0x2C => (write!(w, "SLA H",), 8, 2),
        0x2D => (write!(w, "SLA L",), 8, 2),
        0x2E => (write!(w, "SLA (HL)",), 16, 2),
        // SRL n
        0x3F => (write!(w, "SRL A",), 8, 2),
        0x38 => (write!(w, "SRL B",), 8, 2),
        0x39 => (write!(w, "SRL C",), 8, 2),
        0x3A => (write!(w, "SRL D",), 8, 2),
        0x3B => (write!(w, "SRL E",), 8, 2),
        0x3C => (write!(w, "SRL H",), 8, 2),
        0x3D => (write!(w, "SRL L",), 8, 2),
        0x3E => (write!(w, "SRL (HL)",), 16, 2),
        // BIT b, r
        0x40 => (write!(w, "BIT 0, B",), 8, 2),
        0x41 => (write!(w, "BIT 0, C",), 8, 2),
        0x42 => (write!(w, "BIT 0, D",), 8, 2),
        0x43 => (write!(w, "BIT 0, E",), 8, 2),
        0x44 => (write!(w, "BIT 0, H",), 8, 2),
        0x45 => (write!(w, "BIT 0, L",), 8, 2),
        0x46 => (write!(w, "BIT 0, (HL)",), 12, 2),
        0x47 => (write!(w, "BIT 0, A",), 8, 2),
        0x48 => (write!(w, "BIT 1, B",), 8, 2),
        0x49 => (write!(w, "BIT 1, C",), 8, 2),
        0x4A => (write!(w, "BIT 1, D",), 8, 2),
        0x4B => (write!(w, "BIT 1, E",), 8, 2),
        0x4C => (write!(w, "BIT 1, H",), 8, 2),
        0x4D => (write!(w, "BIT 1, L",), 8, 2),
        0x4E => (write!(w, "BIT 1, (HL)",), 12, 2),
        0x4F => (write!(w, "BIT 1, A",), 8, 2),
        //
        0x50 => (write!(w, "BIT 2, B",), 8, 2),
        0x51 => (write!(w, "BIT 2, C",), 8, 2),
        0x52 => (write!(w, "BIT 2, d",), 8, 2),
        0x53 => (write!(w, "BIT 2, E",), 8, 2),
        0x54 => (write!(w, "BIT 2, H",), 8, 2),
        0x55 => (write!(w, "BIT 2, L",), 8, 2),
        0x56 => (write!(w, "BIT 2, (HL)",), 12, 2),
        0x57 => (write!(w, "BIT 2, A",), 8, 2),
        0x58 => (write!(w, "BIT 3, B",), 8, 2),
        0x59 => (write!(w, "BIT 3, C",), 8, 2),
        0x5A => (write!(w, "BIT 3, D",), 8, 2),
        0x5B => (write!(w, "BIT 3, E",), 8, 2),
        0x5C => (write!(w, "BIT 3, H",), 8, 2),
        0x5D => (write!(w, "BIT 3, L",), 8, 2),
        0x5E => (write!(w, "BIT 3, (HL)",), 12, 2),
        0x5F => (write!(w, "BIT 3, A",), 8, 2),
        //
        0x60 => (write!(w, "BIT 4, B",), 8, 2),
        0x61 => (write!(w, "BIT 4, C",), 8, 2),
        0x62 => (write!(w, "BIT 4, D",), 8, 2),
        0x63 => (write!(w, "BIT 4, E",), 8, 2),
        0x64 => (write!(w, "BIT 4, H",), 8, 2),
        0x65 => (write!(w, "BIT 4, L",), 8, 2),
        0x66 => (write!(w, "BIT 4, (HL)",), 12, 2),
        0x67 => (write!(w, "BIT 4, A",), 8, 2),
        0x68 => (write!(w, "BIT 5, B",), 8, 2),
        0x69 => (write!(w, "BIT 5, C",), 8, 2),
        0x6A => (write!(w, "BIT 5, D",), 8, 2),
        0x6B => (write!(w, "BIT 5, E",), 8, 2),
        0x6C => (write!(w, "BIT 5, H",), 8, 2),
        0x6D => (write!(w, "BIT 5, L",), 8, 2),
        0x6E => (write!(w, "BIT 5, (HL)",), 12, 2),
        0x6F => (write!(w, "BIT 5, A",), 8, 2),
        //
        0x70 => (write!(w, "BIT 6, B",), 8, 2),
        0x71 => (write!(w, "BIT 6, C",), 8, 2),
        0x72 => (write!(w, "BIT 6, D",), 8, 2),
        0x73 => (write!(w, "BIT 6, E",), 8, 2),
        0x74 => (write!(w, "BIT 6, H",), 8, 2),
        0x75 => (write!(w, "BIT 6, L",), 8, 2),
        0x76 => (write!(w, "BIT 6, (HL)",), 12, 2),
        0x77 => (write!(w, "BIT 6, A",), 8, 2),
        0x78 => (write!(w, "BIT 7, B",), 8, 2),
        0x79 => (write!(w, "BIT 7, C",), 8, 2),
        0x7A => (write!(w, "BIT 7, D",), 8, 2),
        0x7B => (write!(w, "BIT 7, E",), 8, 2),
        0x7C => (write!(w, "BIT 7, H",), 8, 2),
        0x7D => (write!(w, "BIT 7, L",), 8, 2),
        0x7E => (write!(w, "BIT 7, (HL)",), 12, 2),
        0x7F => (write!(w, "BIT 7, A",), 8, 2),
        // SET b, r
        0xC0 => (write!(w, "SET 0, B",), 8, 2),
        0xC1 => (write!(w, "SET 0, C",), 8, 2),
        0xC2 => (write!(w, "SET 0, D",), 8, 2),
        0xC3 => (write!(w, "SET 0, E",), 8, 2),
        0xC4 => (write!(w, "SET 0, H",), 8, 2),
        0xC5 => (write!(w, "SET 0, L",), 8, 2),
        0xC6 => (write!(w, "SET 0, (HL)",), 16, 2),
        0xC7 => (write!(w, "SET 0, A",), 8, 2),
        0xC8 => (write!(w, "SET 1, B",), 8, 2),
        0xC9 => (write!(w, "SET 1, C",), 8, 2),
        0xCA => (write!(w, "SET 1, D",), 8, 2),
        0xCB => (write!(w, "SET 1, E",), 8, 2),
        0xCC => (write!(w, "SET 1, H",), 8, 2),
        0xCD => (write!(w, "SET 1, L",), 8, 2),
        0xCE => (write!(w, "SET 1, (HL)",), 16, 2),
        0xCF => (write!(w, "SET 1, A",), 8, 2),
        //
        0xD0 => (write!(w, "SET 2, B",), 8, 2),
        0xD1 => (write!(w, "SET 2, C",), 8, 2),
        0xD2 => (write!(w, "SET 2, D",), 8, 2),
        0xD3 => (write!(w, "SET 2, E",), 8, 2),
        0xD4 => (write!(w, "SET 2, H",), 8, 2),
        0xD5 => (write!(w, "SET 2, L",), 8, 2),
        0xD6 => (write!(w, "SET 2, (HL)",), 16, 2),
        0xD7 => (write!(w, "SET 2, A",), 8, 2),
        0xD8 => (write!(w, "SET 3, B",), 8, 2),
        0xD9 => (write!(w, "SET 3, C",), 8, 2),
        0xDA => (write!(w, "SET 3, D",), 8, 2),
        0xDB => (write!(w, "SET 3, E",), 8, 2),
        0xDC => (write!(w, "SET 3, H",), 8, 2),
        0xDD => (write!(w, "SET 3, L",), 8, 2),
        0xDE => (write!(w, "SET 3, (HL)",), 16, 2),
        0xDF => (write!(w, "SET 3, A",), 8, 2),
        //
        0xE0 => (write!(w, "SET 4, B",), 8, 2),
        0xE1 => (write!(w, "SET 4, C",), 8, 2),
        0xE2 => (write!(w, "SET 4, D",), 8, 2),
        0xE3 => (write!(w, "SET 4, E",), 8, 2),
        0xE4 => (write!(w, "SET 4, H",), 8, 2),
        0xE5 => (write!(w, "SET 4, L",), 8, 2),
        0xE6 => (write!(w, "SET 4, (HL)",), 16, 2),
        0xE7 => (write!(w, "SET 4, A",), 8, 2),
        0xE8 => (write!(w, "SET 5, B",), 8, 2),
        0xE9 => (write!(w, "SET 5, C",), 8, 2),
        0xEA => (write!(w, "SET 5, D",), 8, 2),
        0xEB => (write!(w, "SET 5, E",), 8, 2),
        0xEC => (write!(w, "SET 5, H",), 8, 2),
        0xED => (write!(w, "SET 5, L",), 8, 2),
        0xEE => (write!(w, "SET 5, (HL)",), 16, 2),
        0xEF => (write!(w, "SET 5, A",), 8, 2),
        //
        0xF0 => (write!(w, "SET 6, B",), 8, 2),
        0xF1 => (write!(w, "SET 6, C",), 8, 2),
        0xF2 => (write!(w, "SET 6, D",), 8, 2),
        0xF3 => (write!(w, "SET 6, E",), 8, 2),
        0xF4 => (write!(w, "SET 6, H",), 8, 2),
        0xF5 => (write!(w, "SET 6, L",), 8, 2),
        0xF6 => (write!(w, "SET 6, (HL)",), 16, 2),
        0xF7 => (write!(w, "SET 6, A",), 8, 2),
        0xF8 => (write!(w, "SET 7, B",), 8, 2),
        0xF9 => (write!(w, "SET 7, C",), 8, 2),
        0xFA => (write!(w, "SET 7, D",), 8, 2),
        0xFB => (write!(w, "SET 7, E",), 8, 2),
        0xFC => (write!(w, "SET 7, H",), 8, 2),
        0xFD => (write!(w, "SET 7, L",), 8, 2),
        0xFE => (write!(w, "SET 7, (HL)",), 16, 2),
        0xFF => (write!(w, "SET 7, A",), 8, 2),
        // RES b, r
        0x80 => (write!(w, "RES 0, B",), 8, 2),
        0x81 => (write!(w, "RES 0, C",), 8, 2),
        0x82 => (write!(w, "RES 0, D",), 8, 2),
        0x83 => (write!(w, "RES 0, E",), 8, 2),
        0x84 => (write!(w, "RES 0, H",), 8, 2),
        0x85 => (write!(w, "RES 0, L",), 8, 2),
        0x86 => (write!(w, "RES 0, (HL)",), 16, 2),
        0x87 => (write!(w, "RES 0, A",), 8, 2),
        0x88 => (write!(w, "RES 1, B",), 8, 2),
        0x89 => (write!(w, "RES 1, C",), 8, 2),
        0x8A => (write!(w, "RES 1, D",), 8, 2),
        0x8B => (write!(w, "RES 1, E",), 8, 2),
        0x8C => (write!(w, "RES 1, H",), 8, 2),
        0x8D => (write!(w, "RES 1, L",), 8, 2),
        0x8E => (write!(w, "RES 1, (HL)",), 16, 2),
        0x8F => (write!(w, "RES 1, A",), 8, 2),
        //
        0x90 => (write!(w, "RES 2, B",), 8, 2),
        0x91 => (write!(w, "RES 2, C",), 8, 2),
        0x92 => (write!(w, "RES 2, D",), 8, 2),
        0x93 => (write!(w, "RES 2, E",), 8, 2),
        0x94 => (write!(w, "RES 2, H",), 8, 2),
        0x95 => (write!(w, "RES 2, L",), 8, 2),
        0x96 => (write!(w, "RES 2, (HL)",), 16, 2),
        0x97 => (write!(w, "RES 2, A",), 8, 2),
        0x98 => (write!(w, "RES 3, B",), 8, 2),
        0x99 => (write!(w, "RES 3, C",), 8, 2),
        0x9A => (write!(w, "RES 3, D",), 8, 2),
        0x9B => (write!(w, "RES 3, E",), 8, 2),
        0x9C => (write!(w, "RES 3, H",), 8, 2),
        0x9D => (write!(w, "RES 3, L",), 8, 2),
        0x9E => (write!(w, "RES 3, (HL)",), 16, 2),
        0x9F => (write!(w, "RES 3, A",), 8, 2),
        //
        0xA0 => (write!(w, "RES 4, B",), 8, 2),
        0xA1 => (write!(w, "RES 4, C",), 8, 2),
        0xA2 => (write!(w, "RES 4, D",), 8, 2),
        0xA3 => (write!(w, "RES 4, E",), 8, 2),
        0xA4 => (write!(w, "RES 4, H",), 8, 2),
        0xA5 => (write!(w, "RES 4, L",), 8, 2),
        0xA6 => (write!(w, "RES 4, (HL)",), 16, 2),
        0xA7 => (write!(w, "RES 4, A",), 8, 2),
        0xA8 => (write!(w, "RES 5, B",), 8, 2),
        0xA9 => (write!(w, "RES 5, C",), 8, 2),
        0xAA => (write!(w, "RES 5, D",), 8, 2),
        0xAB => (write!(w, "RES 5, E",), 8, 2),
        0xAC => (write!(w, "RES 5, H",), 8, 2),
        0xAD => (write!(w, "RES 5, L",), 8, 2),
        0xAE => (write!(w, "RES 5, (HL)",), 16, 2),
        0xAF => (write!(w, "RES 5, A",), 8, 2),
        //
        0xB0 => (write!(w, "RES 6, B",), 8, 2),
        0xB1 => (write!(w, "RES 6, C",), 8, 2),
        0xB2 => (write!(w, "RES 6, D",), 8, 2),
        0xB3 => (write!(w, "RES 6, E",), 8, 2),
        0xB4 => (write!(w, "RES 6, H",), 8, 2),
        0xB5 => (write!(w, "RES 6, L",), 8, 2),
        0xB6 => (write!(w, "RES 6, (HL)",), 16, 2),
        0xB7 => (write!(w, "RES 6, A",), 8, 2),
        0xB8 => (write!(w, "RES 7, B",), 8, 2),
        0xB9 => (write!(w, "RES 7, C",), 8, 2),
        0xBA => (write!(w, "RES 7, D",), 8, 2),
        0xBB => (write!(w, "RES 7, E",), 8, 2),
        0xBC => (write!(w, "RES 7, H",), 8, 2),
        0xBD => (write!(w, "RES 7, L",), 8, 2),
        0xBE => (write!(w, "RES 7, (HL)",), 16, 2),
        0xBF => (write!(w, "RES 7, A",), 8, 2),
        // NOT FOUND!
        _ => (write!(w, "Unsupported instruction",), 0, 1),
    };

    if let Ok(_) = result {
        Ok((cycles, bytes))
    } else {
        Err("Unsupported instruction".to_string())
    }
}
/*
fn parse_opcode_cb(b: u16) -> Result<String, String> {
    let test: Vec<Box<dyn FnOnce(u16) -> String>> = match b {
        0 => vec![Box::new(|b: u16| parse_as_u8(b))],
        1 => vec![Box::new(|b: u16| parse_as_u16(b))],
        2 => vec![Box::new(|b: u16| parse_as_i8(b))],
        _ => vec![Box::new(|b: u16| parse_as_i16(b))],
    };

    Ok(write!(w, "".to_string())
}

fn parse_as_u8(word: u16) -> String {
    format!(write!(w, "{:#X}",), word as u8)
}
fn parse_as_u16(word: u16) -> String {
    format!(write!(w, "{:#X}",), word as u16)
}
fn parse_as_i8(word: u16) -> String {
    format!(write!(w, "{:#X}",), word as i8)
}
fn parse_as_i16(word: u16) -> String {
    format!(write!(w, "{:#X}",), word as i16)
}
*/

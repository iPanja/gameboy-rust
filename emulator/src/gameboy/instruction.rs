use std::result;

fn parse_opcode(opcode: u8, next: u8) -> Result<(String, u8, u8), String> {
    let mut r: &str;
    // Vec<Box<dyn FnOnce(u16) -> String>>
    let (result, cycles, bytes) = match opcode {
        // FORMAT 0x00 => { statement ; clock_cycles }
        0xCB => {
            return parse_opcode_cb(opcode);
        }
        //
        // 8-Bit Loads
        //
        // LD nn, n
        // A
        0x06 => ("LD B, n", 8, 2),
        0x0E => ("LD C, n", 8, 2),
        0x16 => ("LD D, n", 8, 2),
        0x1E => ("LD E, n", 8, 2),
        0x26 => ("LD H, n", 8, 2),
        0x2E => ("LD L, n", 8, 2),
        // LD r1, r2
        0x7F => ("LD A, A", 4, 1),
        0x78 => ("LD A, B", 4, 1),
        0x79 => ("LD A, C", 4, 1),
        0x7A => ("LD A, D", 4, 1),
        0x7B => ("LD A, E", 4, 1),
        0x7C => ("LD A, H", 4, 1),
        0x7D => ("LD A, L", 4, 1),
        0x7E => ("LD A, (HL)", 8, 1),
        // B
        0x40 => ("LD B, B", 4, 1),
        0x41 => ("LD B, C", 4, 1),
        0x42 => ("LD B, D", 4, 1),
        0x43 => ("LD B, E", 4, 1),
        0x44 => ("LD B, H", 4, 1),
        0x45 => ("LD B, L", 4, 1),
        0x46 => ("LD B, (HL)", 8, 1),
        // C
        0x48 => ("LD C, B", 4, 1),
        0x49 => ("LD C, C", 4, 1),
        0x4A => ("LD C, D", 4, 1),
        0x4B => ("LD C, E", 4, 1),
        0x4C => ("LD C, H", 4, 1),
        0x4D => ("LD C, L", 4, 1),
        0x4E => ("LD C, (HL)", 8, 1),
        // D
        0x50 => ("LD D, B", 4, 1),
        0x51 => ("LD D, C", 4, 1),
        0x52 => ("LD D, D", 4, 1),
        0x53 => ("LD D, E", 4, 1),
        0x54 => ("LD D, H", 4, 1),
        0x55 => ("LD D, L", 4, 1),
        0x56 => ("LD D, (HL)", 8, 1),
        // E
        0x58 => ("LD E, B", 4, 1),
        0x59 => ("LD E, C", 4, 1),
        0x5A => ("LD E, D", 4, 1),
        0x5B => ("LD E, E", 4, 1),
        0x5C => ("LD E, H", 4, 1),
        0x5D => ("LD E, L", 4, 1),
        0x5E => ("LD E, (HL)", 8, 1),
        // H
        0x60 => ("LD H, B", 4, 1),
        0x61 => ("LD H, C", 4, 1),
        0x62 => ("LD H, D", 4, 1),
        0x63 => ("LD H, E", 4, 1),
        0x64 => ("LD H, H", 4, 1),
        0x65 => ("LD H, L", 4, 1),
        0x66 => ("LD H, (HL)", 8, 1),
        // L
        0x68 => ("LD L, B", 4, 1),
        0x69 => ("LD L, C", 4, 1),
        0x6A => ("LD L, D", 4, 1),
        0x6B => ("LD L, E", 4, 1),
        0x6C => ("LD L, H", 4, 1),
        0x6D => ("LD L, L", 4, 1),
        0x6E => ("LD L, (HL)", 8, 1),
        // (HL)
        0x70 => ("LD (HL), B", 8, 1),
        0x71 => ("LD (HL), C", 8, 1),
        0x72 => ("LD (HL), D", 8, 1),
        0x73 => ("LD (HL), E", 8, 1),
        0x74 => ("LD (HL), H", 8, 1),
        0x75 => ("LD (HL), L", 8, 1),
        0x36 => ("LD (HL), n", 12, 1),
        // LD A, n
        0x78 => ("LD A, B", 4, 1),
        0x7F => ("LD A, A", 4, 1),
        0x79 => ("LD A, C", 4, 1),
        0x7A => ("LD A, D", 4, 1),
        0x7B => ("LD A, E", 4, 1),
        0x7C => ("LD A, H", 4, 1),
        0x7D => ("LD A, L", 4, 1),
        0x0A => ("LD A, (BC)", 4, 1),
        0x1A => ("LD A, (DE)", 4, 1),
        0x7E => ("LD A, (HL)", 4, 1),
        0xFA => ("LD A, (nn)", 4, 3),
        0x3E => ("LD A, #", 4, 2),
        // LD n, A
        0x7F => ("LD A, A", 4, 1),
        0x47 => ("LD B, A", 4, 1),
        0x4F => ("LD C, A", 4, 1),
        0x57 => ("LD D, A", 4, 1),
        0x5F => ("LD E, A", 4, 1),
        0x67 => ("LD H, A", 4, 1),
        0x6F => ("LD L, A", 4, 1),
        0x02 => ("LD (BC), A", 8, 1),
        0x12 => ("LD (DE), A", 8, 1),
        0x77 => ("LD (HL), A", 8, 1),
        0xEA => ("LD (nn), A", 16, 3),
        // LD A, (C)
        0xF2 => ("LD A, ($FF00 + C)", 8, 1),
        // LD (C), A
        0xE2 => ("LD ($FF00 + C), A", 8, 1),
        // LD A, (HLD) ; LD A, (HL-) ; LDD A, (HL) - 0x3A
        0x3A => ("LDD A, (HL-)", 8, 1),
        // LD (HLD), A ; LD (HL-), A ; LDD (HL), A - 0x32
        0x32 => ("LDD (HL-), A", 8, 1),
        // LD A, (HLI) ; LD A, (HL+) ; LDI A, (HL) - 0x2A
        0x2A => ("LDD A, (HL+)", 8, 1),
        // LD (HLI), A ; LD (HL+), A ; LDI (HL), A - 0x22
        0x22 => ("LDD (HL+), A", 8, 1),
        // LDH (n), A - 0xE0
        0xE0 => ("LDH ($FF00 + n), A", 12, 2),
        // LDH A, (n)
        0xF0 => ("LDH A, ($FF00 + n)", 12, 2),
        //
        // 16-Bit Loads
        //
        // LD n, nn
        0x01 => ("LD BC, nn", 12, 3),
        0x11 => ("LD DE, nn", 12, 3),
        0x21 => ("LD HL, nn", 12, 3),
        0x31 => ("LD SP, nn", 12, 3),
        // LD SP, HL
        0xF9 => ("LD SP, HL", 8, 1),
        // LD HL, SP+n
        // LDHL SP, n
        0xF8 => ("LDHL SP, n", 12, 2),
        // LD (nn), SP - 0x08
        0x08 => ("LD (nn), SP", 20, 3),
        // PUSH nn
        0xF5 => ("PUSH AF", 16, 1),
        0xC5 => ("PUSH BC", 16, 1),
        0xD5 => ("PUSH DE", 16, 1),
        0xE5 => ("PUSH HL", 16, 1),
        // POP nn
        0xF1 => ("POP AF", 12, 1),
        0xC1 => ("POP BC", 12, 1),
        0xD1 => ("POP DE", 12, 1),
        0xE1 => ("POP HL", 12, 1),
        //
        //  8-Bit ALU
        //
        // ADD A, n
        0x87 => ("ADD A, A", 4, 1),
        0x80 => ("ADD A, B", 4, 1),
        0x81 => ("ADD A, C", 4, 1),
        0x82 => ("ADD A, D", 4, 1),
        0x83 => ("ADD A, E", 4, 1),
        0x84 => ("ADD A, H", 4, 1),
        0x85 => ("ADD A, L", 4, 1),
        0x86 => ("ADD A, (HL)", 8, 1),
        0xC6 => ("ADD A, n", 8, 2),
        // ADC A, n
        0x8F => ("ADC A, A", 4, 1),
        0x88 => ("ADC A, B", 4, 1),
        0x89 => ("ADC A, C", 4, 1),
        0x8A => ("ADC A, D", 4, 1),
        0x8B => ("ADC A, E", 4, 1),
        0x8C => ("ADC A, H", 4, 1),
        0x8D => ("ADC A, L", 4, 1),
        0x8E => ("ADC A, (HL)", 8, 1),
        0xCE => ("ADC A, n", 8, 2),
        // SUB n
        0x97 => ("SUB A", 4, 1),
        0x90 => ("SUB B", 4, 1),
        0x91 => ("SUB C", 4, 1),
        0x92 => ("SUB D", 4, 1),
        0x93 => ("SUB E", 4, 1),
        0x94 => ("SUB H", 4, 1),
        0x95 => ("SUB L", 4, 1),
        0x96 => ("SUB (HL)", 8, 1),
        0xD6 => ("SUB n", 8, 2),
        // SBC A, n
        0x9F => ("SBC A, A", 4, 1),
        0x98 => ("SBC A, B", 4, 1),
        0x99 => ("SBC A, C", 4, 1),
        0x9A => ("SBC A, D", 4, 1),
        0x9B => ("SBC A, E", 4, 1),
        0x9C => ("SBC A, H", 4, 1),
        0x9D => ("SBC A, L", 4, 1),
        0x9E => ("SBC A, (HL)", 8, 1),
        0xDE => ("SBC A, n", 8, 2),
        // AND n
        0xA7 => ("AND A", 4, 1),
        0xA0 => ("AND B", 4, 1),
        0xA1 => ("AND C", 4, 1),
        0xA2 => ("AND D", 4, 1),
        0xA3 => ("AND E", 4, 1),
        0xA4 => ("AND H", 4, 1),
        0xA5 => ("AND L", 4, 1),
        0xA6 => ("AND (HL)", 8, 1),
        0xE6 => ("AND n", 8, 2),
        // OR n
        0xB7 => ("OR A", 4, 1),
        0xB0 => ("OR B", 4, 1),
        0xB1 => ("OR C", 4, 1),
        0xB2 => ("OR D", 4, 1),
        0xB3 => ("OR E", 4, 1),
        0xB4 => ("OR H", 4, 1),
        0xB5 => ("OR L", 4, 1),
        0xB6 => ("OR (HL)", 8, 1),
        0xF6 => ("OR n", 8, 2),
        // XOR n
        0xAF => ("XOR A", 4, 1),
        0xA8 => ("XOR B", 4, 1),
        0xA9 => ("XOR C", 4, 1),
        0xAA => ("XOR D", 4, 1),
        0xAB => ("XOR E", 4, 1),
        0xAC => ("XOR H", 4, 1),
        0xAD => ("XOR L", 4, 1),
        0xAE => ("XOR (HL)", 8, 1),
        0xEE => ("XOR n", 8, 2),
        // CP n
        0xBF => ("CP A", 4, 1),
        0xB8 => ("CP B", 4, 1),
        0xB9 => ("CP C", 4, 1),
        0xBA => ("CP D", 4, 1),
        0xBB => ("CP E", 4, 1),
        0xBC => ("CP H", 4, 1),
        0xBD => ("CP L", 4, 1),
        0xBE => ("CP (HL)", 8, 1),
        0xFE => ("CP n", 8, 2),
        // INC n
        0x3C => ("INC A", 4, 1),
        0x04 => ("INC B", 4, 1),
        0x0C => ("INC C", 4, 1),
        0x14 => ("INC D", 4, 1),
        0x1C => ("INC E", 4, 1),
        0x24 => ("INC H", 4, 1),
        0x2C => ("INC L", 4, 1),
        0x34 => ("INC (HL)", 12, 1),
        // DEC n
        0x3D => ("DEC A", 4, 1),
        0x05 => ("DEC B", 4, 1),
        0x0D => ("DEC C", 4, 1),
        0x15 => ("DEC D", 4, 1),
        0x1D => ("DEC E", 4, 1),
        0x25 => ("DEC H", 4, 1),
        0x2D => ("DEC L", 4, 1),
        0x35 => ("DEC (HL)", 4, 1),
        //
        // 16-Bit Arithmetic
        //
        // ADD HL, n
        0x09 => ("ADD HL, BC", 8, 1),
        0x19 => ("ADD HL, DE", 8, 1),
        0x29 => ("ADD HL, HL", 8, 1),
        0x39 => ("ADD HL, SP", 8, 1),
        // ADD SP, n
        0xE8 => ("ADD HL, BC", 16, 1), // i8 signed immediate
        // INC nn
        0x03 => ("INC BC", 8, 1),
        0x13 => ("INC DE", 8, 1),
        0x23 => ("INC HL", 8, 1),
        0x33 => ("INC SP", 8, 1),
        // DEC nn
        0x0B => ("DEC BC", 8, 1),
        0x1B => ("DEC DE", 8, 1),
        0x2B => ("DEC HL", 8, 1),
        0x3B => ("DEC SP", 8, 1),
        // DAA
        0x27 => ("DAA", 4, 1),
        // CPL
        0x2F => ("CPL", 4, 1),
        // CCF
        0x3F => ("CCF", 4, 1),
        // SCF
        0x37 => ("SCF", 4, 1),
        // NOP
        0x00 => ("NOP", 4, 1),
        // HALT
        0x76 => ("HALT", 4, 1),
        // STOP
        // DI
        // EI
        //
        // Shifts & Rotations
        //
        // RLCA
        0x07 => ("RLCA", 4, 1),
        // RLA
        0x17 => ("RLA", 4, 1),
        // RRCA
        0x0F => ("RRCA", 4, 1),
        // RRA
        0x1F => ("RRA", 4, 1),
        //
        // Jumps
        //
        // JP nn
        0xC3 => ("JP nn", 12, 3),
        // JP cc, nn
        0xC2 => ("JP NZ, nn", 12, 3),
        0xCA => ("JP Z, nn", 12, 3),
        0xD2 => ("JP NC, nn", 12, 3),
        0xDA => ("JP C, nn", 12, 3),
        // JP (HL)
        0xE9 => ("JP (HL)", 4, 1),
        // JR n
        0x18 => ("JR n", 8, 2),
        // JR cc, n
        0x20 => ("JR NZ, n", 8, 2), // signed i8
        0x28 => ("JR Z, n", 8, 2),
        0x30 => ("JR NC, n", 8, 2),
        0x38 => ("JR C, n", 8, 2),
        //
        // Calls
        //
        // CALL nn
        0xCD => ("CALL NZ, nn", 12, 3), // 2 byte i8 signed
        // CALL cc, nn
        0xC4 => ("CALL NZ, nn", 12, 3), // 2 byte i8 signed
        0xCC => ("CALL Z, nn", 12, 3),
        0xD4 => ("CALL NC, n nn", 12, 3),
        0xDC => ("CALL C, nn", 12, 3),
        //
        // Restarts
        //
        // RST n
        0xC7 => ("RST $00", 16, 1),
        0xCF => ("RST $08", 16, 1),
        0xD7 => ("RST $10", 16, 1),
        0xDF => ("RST $18", 16, 1),
        0xE7 => ("RST $20", 16, 1),
        0xEF => ("RST $28", 16, 1),
        0xF7 => ("RST $30", 16, 1),
        0xFF => ("RST $38", 16, 1),
        //
        // Returns
        //
        // RET
        0xC9 => ("RET", 8, 1),
        // RET cc
        0xC0 => ("RET NZ", 8, 1),
        0xC8 => ("RET Z", 8, 1),
        0xD0 => ("RET NC", 8, 1),
        0xD8 => ("RET C", 8, 1),
        // RETI
        0xD9 => ("RETI", 8, 1),
        //
        // Miscellaneous
        //
        // DI
        0xF3 => ("DI", 4, 1),
        // EI
        0xFB => ("EI", 4, 1),
        // NOT FOUND!
        _ => ("Unsupported instruction", 0, 1),
    };

    if cycles > 0 {
        Ok((result.to_string(), cycles, bytes))
    } else {
        Err(result.to_string())
    }
}

fn parse_opcode_cb(opcode: u8) -> Result<(String, u8, u8), String> {
    let (result, cycles, bytes) = match opcode {
        // MISC
        // SWAP n
        0x37 => ("SWAP A", 8, 2),
        0x30 => ("SWAP B", 8, 2),
        0x31 => ("SWAP C", 8, 2),
        0x32 => ("SWAP D", 8, 2),
        0x33 => ("SWAP E", 8, 2),
        0x34 => ("SWAP H", 8, 2),
        0x35 => ("SWAP L", 8, 2),
        0x36 => ("SWAP (HL)", 16, 2),
        //
        // Shifts & Rotations
        //
        // RLC n
        0x07 => ("RLC A", 8, 2),
        0x00 => ("RLC B", 8, 2),
        0x01 => ("RLC C", 8, 2),
        0x02 => ("RLC D", 8, 2),
        0x03 => ("RLC E", 8, 2),
        0x04 => ("RLC H", 8, 2),
        0x05 => ("RLC L", 8, 2),
        0x06 => ("RLC (HL)", 16, 2),
        // RL n
        0x17 => ("RC A", 8, 2),
        0x10 => ("RC B", 8, 2),
        0x11 => ("RC C", 8, 2),
        0x12 => ("RC D", 8, 2),
        0x13 => ("RC E", 8, 2),
        0x14 => ("RC H", 8, 2),
        0x15 => ("RC L", 8, 2),
        0x16 => ("RC (HL)", 16, 2),
        // RRC n
        0x0F => ("RRC A", 8, 2),
        0x08 => ("RRC B", 8, 2),
        0x09 => ("RRC C", 8, 2),
        0x0A => ("RRC D", 8, 2),
        0x0B => ("RRC E", 8, 2),
        0x0C => ("RRC H", 8, 2),
        0x0D => ("RRC L", 8, 2),
        0x0E => ("RRC (HL)", 16, 2),
        // RR n
        0x1F => ("RR A", 8, 2),
        0x18 => ("RR B", 8, 2),
        0x19 => ("RR C", 8, 2),
        0x1A => ("RR D", 8, 2),
        0x1B => ("RR E", 8, 2),
        0x1C => ("RR H", 8, 2),
        0x1D => ("RR L", 8, 2),
        0x1E => ("RR (HL)", 16, 2),
        // SLA n
        0x27 => ("SLA A", 8, 2),
        0x20 => ("SLA B", 8, 2),
        0x21 => ("SLA C", 8, 2),
        0x22 => ("SLA D", 8, 2),
        0x23 => ("SLA E", 8, 2),
        0x24 => ("SLA H", 8, 2),
        0x25 => ("SLA L", 8, 2),
        0x26 => ("SLA (HL)", 16, 2),
        // SRA n
        0x2F => ("SLA A", 8, 2),
        0x28 => ("SLA B", 8, 2),
        0x29 => ("SLA C", 8, 2),
        0x2A => ("SLA D", 8, 2),
        0x2B => ("SLA E", 8, 2),
        0x2C => ("SLA H", 8, 2),
        0x2D => ("SLA L", 8, 2),
        0x2E => ("SLA (HL)", 16, 2),
        // SRL n
        0x3F => ("SRL A", 8, 2),
        0x38 => ("SRL B", 8, 2),
        0x39 => ("SRL C", 8, 2),
        0x3A => ("SRL D", 8, 2),
        0x3B => ("SRL E", 8, 2),
        0x3C => ("SRL H", 8, 2),
        0x3D => ("SRL L", 8, 2),
        0x3E => ("SRL (HL)", 16, 2),
        // BIT b, r
        0x40 => ("BIT 0, B", 8, 2),
        0x41 => ("BIT 0, C", 8, 2),
        0x42 => ("BIT 0, D", 8, 2),
        0x43 => ("BIT 0, E", 8, 2),
        0x44 => ("BIT 0, H", 8, 2),
        0x45 => ("BIT 0, L", 8, 2),
        0x46 => ("BIT 0, (HL)", 12, 2),
        0x47 => ("BIT 0, A", 8, 2),
        0x48 => ("BIT 1, B", 8, 2),
        0x49 => ("BIT 1, C", 8, 2),
        0x4A => ("BIT 1, D", 8, 2),
        0x4B => ("BIT 1, E", 8, 2),
        0x4C => ("BIT 1, H", 8, 2),
        0x4D => ("BIT 1, L", 8, 2),
        0x4E => ("BIT 1, (HL)", 12, 2),
        0x4F => ("BIT 1, A", 8, 2),
        //
        0x50 => ("BIT 2, B", 8, 2),
        0x51 => ("BIT 2, C", 8, 2),
        0x52 => ("BIT 2, d", 8, 2),
        0x53 => ("BIT 2, E", 8, 2),
        0x54 => ("BIT 2, H", 8, 2),
        0x55 => ("BIT 2, L", 8, 2),
        0x56 => ("BIT 2, (HL)", 12, 2),
        0x57 => ("BIT 2, A", 8, 2),
        0x58 => ("BIT 3, B", 8, 2),
        0x59 => ("BIT 3, C", 8, 2),
        0x5A => ("BIT 3, D", 8, 2),
        0x5B => ("BIT 3, E", 8, 2),
        0x5C => ("BIT 3, H", 8, 2),
        0x5D => ("BIT 3, L", 8, 2),
        0x5E => ("BIT 3, (HL)", 12, 2),
        0x5F => ("BIT 3, A", 8, 2),
        //
        0x60 => ("BIT 4, B", 8, 2),
        0x61 => ("BIT 4, C", 8, 2),
        0x62 => ("BIT 4, D", 8, 2),
        0x63 => ("BIT 4, E", 8, 2),
        0x64 => ("BIT 4, H", 8, 2),
        0x65 => ("BIT 4, L", 8, 2),
        0x66 => ("BIT 4, (HL)", 12, 2),
        0x67 => ("BIT 4, A", 8, 2),
        0x68 => ("BIT 5, B", 8, 2),
        0x69 => ("BIT 5, C", 8, 2),
        0x6A => ("BIT 5, D", 8, 2),
        0x6B => ("BIT 5, E", 8, 2),
        0x6C => ("BIT 5, H", 8, 2),
        0x6D => ("BIT 5, L", 8, 2),
        0x6E => ("BIT 5, (HL)", 12, 2),
        0x6F => ("BIT 5, A", 8, 2),
        //
        0x70 => ("BIT 6, B", 8, 2),
        0x71 => ("BIT 6, C", 8, 2),
        0x72 => ("BIT 6, D", 8, 2),
        0x73 => ("BIT 6, E", 8, 2),
        0x74 => ("BIT 6, H", 8, 2),
        0x75 => ("BIT 6, L", 8, 2),
        0x76 => ("BIT 6, (HL)", 12, 2),
        0x77 => ("BIT 6, A", 8, 2),
        0x78 => ("BIT 7, B", 8, 2),
        0x79 => ("BIT 7, C", 8, 2),
        0x7A => ("BIT 7, D", 8, 2),
        0x7B => ("BIT 7, E", 8, 2),
        0x7C => ("BIT 7, H", 8, 2),
        0x7D => ("BIT 7, L", 8, 2),
        0x7E => ("BIT 7, (HL)", 12, 2),
        0x7F => ("BIT 7, A", 8, 2),
        // SET b, r
        0xC0 => ("SET 0, B", 8, 2),
        0xC1 => ("SET 0, C", 8, 2),
        0xC2 => ("SET 0, D", 8, 2),
        0xC3 => ("SET 0, E", 8, 2),
        0xC4 => ("SET 0, H", 8, 2),
        0xC5 => ("SET 0, L", 8, 2),
        0xC6 => ("SET 0, (HL)", 16, 2),
        0xC7 => ("SET 0, A", 8, 2),
        0xC8 => ("SET 1, B", 8, 2),
        0xC9 => ("SET 1, C", 8, 2),
        0xCA => ("SET 1, D", 8, 2),
        0xCB => ("SET 1, E", 8, 2),
        0xCC => ("SET 1, H", 8, 2),
        0xCD => ("SET 1, L", 8, 2),
        0xCE => ("SET 1, (HL)", 16, 2),
        0xCF => ("SET 1, A", 8, 2),
        //
        0xD0 => ("SET 2, B", 8, 2),
        0xD1 => ("SET 2, C", 8, 2),
        0xD2 => ("SET 2, D", 8, 2),
        0xD3 => ("SET 2, E", 8, 2),
        0xD4 => ("SET 2, H", 8, 2),
        0xD5 => ("SET 2, L", 8, 2),
        0xD6 => ("SET 2, (HL)", 16, 2),
        0xD7 => ("SET 2, A", 8, 2),
        0xD8 => ("SET 3, B", 8, 2),
        0xD9 => ("SET 3, C", 8, 2),
        0xDA => ("SET 3, D", 8, 2),
        0xDB => ("SET 3, E", 8, 2),
        0xDC => ("SET 3, H", 8, 2),
        0xDD => ("SET 3, L", 8, 2),
        0xDE => ("SET 3, (HL)", 16, 2),
        0xDF => ("SET 3, A", 8, 2),
        //
        0xE0 => ("SET 4, B", 8, 2),
        0xE1 => ("SET 4, C", 8, 2),
        0xE2 => ("SET 4, D", 8, 2),
        0xE3 => ("SET 4, E", 8, 2),
        0xE4 => ("SET 4, H", 8, 2),
        0xE5 => ("SET 4, L", 8, 2),
        0xE6 => ("SET 4, (HL)", 16, 2),
        0xE7 => ("SET 4, A", 8, 2),
        0xE8 => ("SET 5, B", 8, 2),
        0xE9 => ("SET 5, C", 8, 2),
        0xEA => ("SET 5, D", 8, 2),
        0xEB => ("SET 5, E", 8, 2),
        0xEC => ("SET 5, H", 8, 2),
        0xED => ("SET 5, L", 8, 2),
        0xEE => ("SET 5, (HL)", 16, 2),
        0xEF => ("SET 5, A", 8, 2),
        //
        0xF0 => ("SET 6, B", 8, 2),
        0xF1 => ("SET 6, C", 8, 2),
        0xF2 => ("SET 6, D", 8, 2),
        0xF3 => ("SET 6, E", 8, 2),
        0xF4 => ("SET 6, H", 8, 2),
        0xF5 => ("SET 6, L", 8, 2),
        0xF6 => ("SET 6, (HL)", 16, 2),
        0xF7 => ("SET 6, A", 8, 2),
        0xF8 => ("SET 7, B", 8, 2),
        0xF9 => ("SET 7, C", 8, 2),
        0xFA => ("SET 7, D", 8, 2),
        0xFB => ("SET 7, E", 8, 2),
        0xFC => ("SET 7, H", 8, 2),
        0xFD => ("SET 7, L", 8, 2),
        0xFE => ("SET 7, (HL)", 16, 2),
        0xFF => ("SET 7, A", 8, 2),
        // RES b, r
        0x80 => ("RES 0, B", 8, 2),
        0x81 => ("RES 0, C", 8, 2),
        0x82 => ("RES 0, D", 8, 2),
        0x83 => ("RES 0, E", 8, 2),
        0x84 => ("RES 0, H", 8, 2),
        0x85 => ("RES 0, L", 8, 2),
        0x86 => ("RES 0, (HL)", 16, 2),
        0x87 => ("RES 0, A", 8, 2),
        0x88 => ("RES 1, B", 8, 2),
        0x89 => ("RES 1, C", 8, 2),
        0x8A => ("RES 1, D", 8, 2),
        0x8B => ("RES 1, E", 8, 2),
        0x8C => ("RES 1, H", 8, 2),
        0x8D => ("RES 1, L", 8, 2),
        0x8E => ("RES 1, (HL)", 16, 2),
        0x8F => ("RES 1, A", 8, 2),
        //
        0x90 => ("RES 2, B", 8, 2),
        0x91 => ("RES 2, C", 8, 2),
        0x92 => ("RES 2, D", 8, 2),
        0x93 => ("RES 2, E", 8, 2),
        0x94 => ("RES 2, H", 8, 2),
        0x95 => ("RES 2, L", 8, 2),
        0x96 => ("RES 2, (HL)", 16, 2),
        0x97 => ("RES 2, A", 8, 2),
        0x98 => ("RES 3, B", 8, 2),
        0x99 => ("RES 3, C", 8, 2),
        0x9A => ("RES 3, D", 8, 2),
        0x9B => ("RES 3, E", 8, 2),
        0x9C => ("RES 3, H", 8, 2),
        0x9D => ("RES 3, L", 8, 2),
        0x9E => ("RES 3, (HL)", 16, 2),
        0x9F => ("RES 3, A", 8, 2),
        //
        0xA0 => ("RES 4, B", 8, 2),
        0xA1 => ("RES 4, C", 8, 2),
        0xA2 => ("RES 4, D", 8, 2),
        0xA3 => ("RES 4, E", 8, 2),
        0xA4 => ("RES 4, H", 8, 2),
        0xA5 => ("RES 4, L", 8, 2),
        0xA6 => ("RES 4, (HL)", 16, 2),
        0xA7 => ("RES 4, A", 8, 2),
        0xA8 => ("RES 5, B", 8, 2),
        0xA9 => ("RES 5, C", 8, 2),
        0xAA => ("RES 5, D", 8, 2),
        0xAB => ("RES 5, E", 8, 2),
        0xAC => ("RES 5, H", 8, 2),
        0xAD => ("RES 5, L", 8, 2),
        0xAE => ("RES 5, (HL)", 16, 2),
        0xAF => ("RES 5, A", 8, 2),
        //
        0xB0 => ("RES 6, B", 8, 2),
        0xB1 => ("RES 6, C", 8, 2),
        0xB2 => ("RES 6, D", 8, 2),
        0xB3 => ("RES 6, E", 8, 2),
        0xB4 => ("RES 6, H", 8, 2),
        0xB5 => ("RES 6, L", 8, 2),
        0xB6 => ("RES 6, (HL)", 16, 2),
        0xB7 => ("RES 6, A", 8, 2),
        0xB8 => ("RES 7, B", 8, 2),
        0xB9 => ("RES 7, C", 8, 2),
        0xBA => ("RES 7, D", 8, 2),
        0xBB => ("RES 7, E", 8, 2),
        0xBC => ("RES 7, H", 8, 2),
        0xBD => ("RES 7, L", 8, 2),
        0xBE => ("RES 7, (HL)", 16, 2),
        0xBF => ("RES 7, A", 8, 2),
        // NOT FOUND!
        _ => ("Unsupported instruction", 0, 1),
    };

    if cycles > 0 {
        Ok((result.to_string(), cycles, bytes))
    } else {
        Err(result.to_string())
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

    Ok("".to_string())
}

fn parse_as_u8(word: u16) -> String {
    format!("{:#X}", word as u8)
}
fn parse_as_u16(word: u16) -> String {
    format!("{:#X}", word as u16)
}
fn parse_as_i8(word: u16) -> String {
    format!("{:#X}", word as i8)
}
fn parse_as_i16(word: u16) -> String {
    format!("{:#X}", word as i16)
}
*/

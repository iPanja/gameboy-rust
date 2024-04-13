use super::register;
use super::Bus;
use super::Flag;
use super::Registers;

const IS_DEBUGGING: bool = false;

pub struct CPU {
    registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let opcode = bus.ram_read_byte(self.registers.pc);
        //println!("instruction {:#X}: {:#X}", self.registers.pc, opcode);
        self.registers.pc += 1;

        let cycles = match opcode {
            // FORMAT 0x00 => { statement ; clock_cycles }
            //
            // 8-Bit Loads
            //
            /// LD nn, n
            0xCB => self.step_cb(bus),
            0x06 => {
                self.registers.b = self.read_byte(bus);
                8
            }
            0x0E => {
                self.registers.c = self.read_byte(bus);
                8
            }
            0x16 => {
                self.registers.d = self.read_byte(bus);
                8
            }
            0x1E => {
                self.registers.e = self.read_byte(bus);
                8
            }
            0x26 => {
                self.registers.h = self.read_byte(bus);
                8
            }
            0x2E => {
                self.registers.l = self.read_byte(bus);
                8
            }
            /// LD r1, r2
            //// A
            0x7F => {
                self.registers.a = self.registers.a;
                4
            }
            0x78 => {
                self.registers.a = self.registers.b;
                4
            }
            0x79 => {
                self.registers.a = self.registers.c;
                4
            }
            0x7A => {
                self.registers.a = self.registers.d;
                4
            }
            0x7B => {
                self.registers.a = self.registers.e;
                4
            }
            0x7C => {
                self.registers.a = self.registers.h;
                4
            }
            0x7D => {
                self.registers.a = self.registers.l;
                4
            }
            0x7E => {
                self.registers.a = self.read_ram_at_hl(bus);
                8
            }
            //// B
            0x40 => {
                self.registers.b = self.registers.b;
                4
            }
            0x41 => {
                self.registers.b = self.registers.c;
                4
            }
            0x42 => {
                self.registers.b = self.registers.d;
                4
            }
            0x43 => {
                self.registers.b = self.registers.e;
                4
            }
            0x44 => {
                self.registers.b = self.registers.h;
                4
            }
            0x45 => {
                self.registers.b = self.registers.l;
                4
            }
            0x46 => {
                self.registers.b = self.read_ram_at_hl(bus);
                8
            }
            //// C
            0x48 => {
                self.registers.c = self.registers.b;
                4
            }
            0x49 => {
                self.registers.c = self.registers.c;
                4
            }
            0x4A => {
                self.registers.c = self.registers.d;
                4
            }
            0x4B => {
                self.registers.c = self.registers.e;
                4
            }
            0x4C => {
                self.registers.c = self.registers.h;
                4
            }
            0x4D => {
                self.registers.c = self.registers.l;
                4
            }
            0x4E => {
                self.registers.c = self.read_ram_at_hl(bus);
                8
            }
            //// D
            0x50 => {
                self.registers.d = self.registers.b;
                4
            }
            0x51 => {
                self.registers.d = self.registers.c;
                4
            }
            0x52 => {
                self.registers.d = self.registers.d;
                4
            }
            0x53 => {
                self.registers.d = self.registers.e;
                4
            }
            0x54 => {
                self.registers.d = self.registers.h;
                4
            }
            0x55 => {
                self.registers.d = self.registers.l;
                4
            }
            0x56 => {
                self.registers.d = self.read_ram_at_hl(bus);
                8
            }
            //// E
            0x58 => {
                self.registers.e = self.registers.b;
                4
            }
            0x59 => {
                self.registers.e = self.registers.c;
                4
            }
            0x5A => {
                self.registers.e = self.registers.d;
                4
            }
            0x5B => {
                self.registers.e = self.registers.e;
                4
            }
            0x5C => {
                self.registers.e = self.registers.h;
                4
            }
            0x5D => {
                self.registers.e = self.registers.l;
                4
            }
            0x5E => {
                self.registers.e = self.read_ram_at_hl(bus);
                8
            }
            //// H
            0x60 => {
                self.registers.h = self.registers.b;
                4
            }
            0x61 => {
                self.registers.h = self.registers.c;
                4
            }
            0x62 => {
                self.registers.h = self.registers.d;
                4
            }
            0x63 => {
                self.registers.h = self.registers.e;
                4
            }
            0x64 => {
                self.registers.h = self.registers.h;
                4
            }
            0x65 => {
                self.registers.h = self.registers.l;
                4
            }
            0x66 => {
                self.registers.h = self.read_ram_at_hl(bus);
                8
            }
            //// L
            0x68 => {
                self.registers.l = self.registers.b;
                4
            }
            0x69 => {
                self.registers.l = self.registers.c;
                4
            }
            0x6A => {
                self.registers.l = self.registers.d;
                4
            }
            0x6B => {
                self.registers.l = self.registers.e;
                4
            }
            0x6C => {
                self.registers.l = self.registers.h;
                4
            }
            0x6D => {
                self.registers.l = self.registers.l;
                4
            }
            0x6E => {
                self.registers.l = self.read_ram_at_hl(bus);
                8
            }
            //// (HL)
            0x70 => {
                self.write_ram_at_hl(bus, self.registers.b);
                8
            }
            0x71 => {
                self.write_ram_at_hl(bus, self.registers.c);
                8
            }
            0x72 => {
                self.write_ram_at_hl(bus, self.registers.d);
                8
            }
            0x73 => {
                self.write_ram_at_hl(bus, self.registers.e);
                8
            }
            0x74 => {
                self.write_ram_at_hl(bus, self.registers.h);
                8
            }
            0x75 => {
                self.write_ram_at_hl(bus, self.registers.l);
                8
            }
            0x36 => {
                let byte = self.read_byte(bus);
                self.write_ram_at_hl(bus, byte);
                12
            }
            /// LD A, n
            0x7F => {
                self.registers.a = self.registers.a;
                4
            }
            0x78 => {
                self.registers.a = self.registers.b;
                4
            }
            0x79 => {
                self.registers.a = self.registers.c;
                4
            }
            0x7A => {
                self.registers.a = self.registers.d;
                4
            }
            0x7B => {
                self.registers.a = self.registers.e;
                4
            }
            0x7C => {
                self.registers.a = self.registers.h;
                4
            }
            0x7D => {
                self.registers.a = self.registers.l;
                4
            }
            0x0A => {
                self.registers.a = bus.ram_read_byte(self.registers.get_bc());
                8
            }
            0x1A => {
                self.registers.a = bus.ram_read_byte(self.registers.get_de());
                8
            }
            0x7E => {
                self.registers.a = bus.ram_read_byte(self.registers.get_hl());
                8
            }
            0xFA => {
                self.registers.a = bus.ram_read_byte(self.read_word(bus));
                16
            }
            0x3E => {
                self.registers.a = self.read_byte(bus);
                8
            }
            /// LD n, A
            0x7F => {
                self.registers.a = self.registers.a;
                4
            }
            0x47 => {
                self.registers.b = self.registers.a;
                4
            }
            0x4F => {
                self.registers.c = self.registers.a;
                4
            }
            0x57 => {
                self.registers.d = self.registers.a;
                4
            }
            0x5F => {
                self.registers.e = self.registers.a;
                4
            }
            0x67 => {
                self.registers.h = self.registers.a;
                4
            }
            0x6F => {
                self.registers.l = self.registers.a;
                4
            }
            0x02 => {
                bus.ram_write_byte(self.registers.get_bc(), self.registers.a);
                8
            }
            0x12 => {
                bus.ram_write_byte(self.registers.get_de(), self.registers.a);
                8
            }
            0x77 => {
                bus.ram_write_byte(self.registers.get_hl(), self.registers.a);
                8
            }
            0xEA => {
                bus.ram_write_byte(self.read_word(bus), self.registers.a);
                16
            }
            /// LD A, (C)
            0xF2 => {
                let addr = 0xFF00 | (self.registers.c as u16);
                let byte = bus.ram_read_byte(addr);
                self.registers.a = byte;
                8
            }
            /// LD (C), A
            0xE2 => {
                let addr = 0xFF00 | (self.registers.c as u16);
                bus.ram_write_byte(addr, self.registers.a);
                8
            }
            /// LD A, (HLD) ; LD A, (HL-) ; LDD A, (HL) - 0x3A
            0x3A => {
                let byte = bus.ram_read_byte(self.registers.hld());
                self.registers.a = byte;
                8
            }
            /// LD (HLD), A ; LD (HL-), A ; LDD (HL), A - 0x32
            0x32 => {
                let byte = self.registers.a;
                bus.ram_write_byte(self.registers.hld(), byte);
                8
            }
            /// LD A, (HLI) ; LD A, (HL+) ; LDI A, (HL) - 0x2A
            0x2A => {
                let byte = bus.ram_read_byte(self.registers.hli());
                self.registers.a = byte;
                8
            }
            /// LD (HLI), A ; LD (HL+), A ; LDI (HL), A - 0x22
            0x22 => {
                let byte = self.registers.a;
                bus.ram_write_byte(self.registers.hli(), byte);
                8
            }
            /// LDH (n), A - 0xE0
            0xE0 => {
                bus.ram_write_byte(0xFF00 | (self.read_byte(bus) as u16), self.registers.a);
                12
            }
            /// LDH A, (n)
            0xF0 => {
                self.registers.a = bus.ram_read_byte(0xFF00 | (self.read_byte(bus) as u16));
                12
            }
            //
            // 16-Bit Loads
            //
            /// LD n, nn
            0x01 => {
                let word = self.read_word(bus);
                self.registers.set_bc(word);
                12
            }
            0x11 => {
                let word = self.read_word(bus);
                self.registers.set_de(word);
                12
            }
            0x21 => {
                let word = self.read_word(bus);
                self.registers.set_hl(word);
                12
            }
            0x31 => {
                self.registers.sp = self.read_word(bus);
                12
            }
            /// LD SP, HL
            0xF9 => {
                self.registers.sp = self.registers.get_hl();
                8
            }
            /// LD HL, SP+n
            /// LDHL SP, n
            0xF8 => {
                let sum = self.registers.sp + (self.read_byte(bus) as u16);
                self.registers.set_hl(sum);
                12
            }
            /// LD (nn), SP - 0x08
            0x08 => {
                let addr = self.read_word(bus);
                bus.ram_write_word(addr, self.registers.sp);
                20
            }
            /// PUSH nn
            0xF5 => {
                self.stack_push(bus, self.registers.get_af());
                16
            }
            0xC5 => {
                self.stack_push(bus, self.registers.get_bc());
                16
            }
            0xD5 => {
                self.stack_push(bus, self.registers.get_de());
                16
            }
            0xE5 => {
                self.stack_push(bus, self.registers.get_hl());
                16
            }
            /// POP nn
            0xF1 => {
                let word = self.read_word(bus);
                self.registers.set_af(word);
                12
            }
            0xC1 => {
                let word = self.read_word(bus);
                self.registers.set_bc(word);
                12
            }
            0xD1 => {
                let word = self.read_word(bus);
                self.registers.set_de(word);
                12
            }
            0xE1 => {
                let word = self.read_word(bus);
                self.registers.set_hl(word);
                12
            }
            //
            //  8-Bit ALU
            //
            /// ADD A, n
            0x87 => {
                self.alu_add(self.registers.a, false);
                4
            }
            0x80 => {
                self.alu_add(self.registers.b, false);
                4
            }
            0x81 => {
                self.alu_add(self.registers.c, false);
                4
            }
            0x82 => {
                self.alu_add(self.registers.d, false);
                4
            }
            0x83 => {
                self.alu_add(self.registers.e, false);
                4
            }
            0x84 => {
                self.alu_add(self.registers.h, false);
                4
            }
            0x85 => {
                self.alu_add(self.registers.l, false);
                4
            }
            0x86 => {
                self.alu_add(self.read_ram_at_hl(bus), false);
                8
            }
            0xC6 => {
                let byte = self.read_byte(bus);
                self.alu_add(byte, false);
                8
            }
            /// ADC A, n
            0x8F => {
                self.alu_add(self.registers.a, true);
                4
            }
            0x88 => {
                self.alu_add(self.registers.b, true);
                4
            }
            0x89 => {
                self.alu_add(self.registers.c, true);
                4
            }
            0x8A => {
                self.alu_add(self.registers.d, true);
                4
            }
            0x8B => {
                self.alu_add(self.registers.e, true);
                4
            }
            0x8C => {
                self.alu_add(self.registers.h, true);
                4
            }
            0x8D => {
                self.alu_add(self.registers.l, true);
                4
            }
            0x8E => {
                self.alu_add(self.read_ram_at_hl(bus), true);
                8
            }
            0xCE => {
                let byte = self.read_byte(bus);
                self.alu_add(byte, true);
                8
            }
            /// SUB n
            0x97 => {
                self.alu_sub(self.registers.a, false);
                4
            }
            0x90 => {
                self.alu_sub(self.registers.b, false);
                4
            }
            0x91 => {
                self.alu_sub(self.registers.c, false);
                4
            }
            0x92 => {
                self.alu_sub(self.registers.d, false);
                4
            }
            0x93 => {
                self.alu_sub(self.registers.e, false);
                4
            }
            0x94 => {
                self.alu_sub(self.registers.h, false);
                4
            }
            0x95 => {
                self.alu_sub(self.registers.l, false);
                4
            }
            0x96 => {
                self.alu_sub(self.read_ram_at_hl(bus), false);
                8
            }
            0xD6 => {
                let byte = self.read_byte(bus);
                self.alu_sub(byte, false);
                8
            }
            /// SBC A, n
            0x9F => {
                self.alu_sub(self.registers.a, true);
                4
            }
            0x98 => {
                self.alu_sub(self.registers.b, true);
                4
            }
            0x99 => {
                self.alu_sub(self.registers.c, true);
                4
            }
            0x9A => {
                self.alu_sub(self.registers.d, true);
                4
            }
            0x9B => {
                self.alu_sub(self.registers.e, true);
                4
            }
            0x9C => {
                self.alu_sub(self.registers.h, true);
                4
            }
            0x9D => {
                self.alu_sub(self.registers.l, true);
                4
            }
            0x9E => {
                self.alu_sub(self.read_ram_at_hl(bus), true);
                8
            }
            //0x?? => {self.alu_sub(self.read_byte(bus), true);}
            /// AND n
            0xA7 => {
                self.alu_and(self.registers.a);
                4
            }
            0xA0 => {
                self.alu_and(self.registers.b);
                4
            }
            0xA1 => {
                self.alu_and(self.registers.c);
                4
            }
            0xA2 => {
                self.alu_and(self.registers.d);
                4
            }
            0xA3 => {
                self.alu_and(self.registers.e);
                4
            }
            0xA4 => {
                self.alu_and(self.registers.h);
                4
            }
            0xA5 => {
                self.alu_and(self.registers.l);
                4
            }
            0xA6 => {
                self.alu_and(self.read_ram_at_hl(bus));
                8
            }
            0xE6 => {
                let byte = self.read_byte(bus);
                self.alu_and(byte);
                8
            }
            /// OR n
            0xB7 => {
                self.alu_or(self.registers.a);
                4
            }
            0xB0 => {
                self.alu_or(self.registers.b);
                4
            }
            0xB1 => {
                self.alu_or(self.registers.c);
                4
            }
            0xB2 => {
                self.alu_or(self.registers.d);
                4
            }
            0xB3 => {
                self.alu_or(self.registers.e);
                4
            }
            0xB4 => {
                self.alu_or(self.registers.h);
                4
            }
            0xB5 => {
                self.alu_or(self.registers.l);
                4
            }
            0xB6 => {
                self.alu_or(self.read_ram_at_hl(bus));
                8
            }
            0xF6 => {
                let byte = self.read_byte(bus);
                self.alu_or(byte);
                8
            }
            /// XOR n
            0xAF => {
                self.alu_xor(self.registers.a);
                4
            }
            0xA8 => {
                self.alu_xor(self.registers.b);
                4
            }
            0xA9 => {
                self.alu_xor(self.registers.c);
                4
            }
            0xAA => {
                self.alu_xor(self.registers.d);
                4
            }
            0xAB => {
                self.alu_xor(self.registers.e);
                4
            }
            0xAC => {
                self.alu_xor(self.registers.h);
                4
            }
            0xAD => {
                self.alu_xor(self.registers.l);
                4
            }
            0xAE => {
                self.alu_xor(self.read_ram_at_hl(bus));
                8
            }
            0xEE => {
                let byte = self.read_byte(bus);
                self.alu_xor(byte);
                8
            }
            /// CP n
            0xBF => {
                self.alu_cp(self.registers.a);
                4
            }
            0xB8 => {
                self.alu_cp(self.registers.b);
                4
            }
            0xB9 => {
                self.alu_cp(self.registers.c);
                4
            }
            0xBA => {
                self.alu_cp(self.registers.d);
                4
            }
            0xBB => {
                self.alu_cp(self.registers.e);
                4
            }
            0xBC => {
                self.alu_cp(self.registers.h);
                4
            }
            0xBD => {
                self.alu_cp(self.registers.l);
                4
            }
            0xBE => {
                self.alu_cp(self.read_ram_at_hl(bus));
                8
            }
            0xFE => {
                let byte = self.read_byte(bus);
                self.alu_cp(byte);
                8
            }
            /// INC n
            0x3C => {
                self.registers.a = self.inc(self.registers.a);
                4
            }
            0x04 => {
                self.registers.b = self.inc(self.registers.b);
                4
            }
            0x0C => {
                self.registers.c = self.inc(self.registers.c);
                4
            }
            0x14 => {
                self.registers.d = self.inc(self.registers.d);
                4
            }
            0x1C => {
                self.registers.e = self.inc(self.registers.e);
                4
            }
            0x24 => {
                self.registers.h = self.inc(self.registers.h);
                4
            }
            0x2C => {
                self.registers.l = self.inc(self.registers.l);
                4
            }
            0x34 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.inc(byte);
                self.write_ram_at_hl(bus, result);
                12
            }
            /// DEC n
            0x3D => {
                self.registers.a = self.dec(self.registers.a);
                4
            }
            0x05 => {
                self.registers.b = self.dec(self.registers.b);
                4
            }
            0x0D => {
                self.registers.c = self.dec(self.registers.c);
                4
            }
            0x15 => {
                self.registers.d = self.dec(self.registers.d);
                4
            }
            0x1D => {
                self.registers.e = self.dec(self.registers.e);
                4
            }
            0x25 => {
                self.registers.h = self.dec(self.registers.h);
                4
            }
            0x2D => {
                self.registers.l = self.dec(self.registers.l);
                4
            }
            0x35 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.dec(byte);
                self.write_ram_at_hl(bus, result);
                12
            }
            //
            // 16-Bit Arithmetic
            //
            /// ADD HL, n
            0x09 => {
                let word = self.registers.get_bc();
                self.alu_add_16(word);
                8
            }
            0x19 => {
                let word = self.registers.get_de();
                self.alu_add_16(word);
                8
            }
            0x29 => {
                let word = self.registers.get_hl();
                self.alu_add_16(word);
                8
            }
            0x39 => {
                self.alu_add_16(self.registers.sp);
                8
            }
            /// ADD SP, n
            0xE8 => {
                let byte = self.read_byte(bus);
                self.registers.sp = self.alu_add_16_imm(self.registers.sp, byte);
                16
            }
            /// INC nn
            0x03 => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                8
            }
            0x13 => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_add(1));
                8
            }
            0x23 => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                8
            }
            0x33 => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
                8
            }
            /// DEC nn
            0x0B => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                8
            }
            0x1B => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_add(1));
                8
            }
            0x2B => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                8
            }
            0x3B => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
                8
            }
            // DAA
            // CPL
            // CCF
            // SCF
            0x00 => 4,
            // HALT
            // STOP
            // DI
            // EI
            //
            // Shifts & Rotations
            //
            /// RLCA
            0x07 => {
                self.registers.a = self.rotate_circular(self.registers.a, true);
                4
            }
            /// RLA
            0x17 => {
                self.registers.a = self.rotate(self.registers.a, true);
                4
            }
            /// RRCA
            0x0F => {
                self.registers.a = self.rotate_circular(self.registers.a, false);
                4
            }
            /// RRA
            0x1F => {
                self.registers.a = self.rotate(self.registers.a, false);
                4
            }
            //
            // Jumps
            //
            /// JP nn
            0xC3 => {
                let addr = self.read_word(bus);
                self.jump_to(addr);
                12
            }
            /// JP cc, nn
            0xC2 => {
                let addr = self.read_word(bus);
                if !self.registers.f.zero {
                    self.jump_to(addr);
                }
                12
            }
            0xCA => {
                let addr = self.read_word(bus);
                if self.registers.f.zero {
                    self.jump_to(addr);
                }
                12
            }
            0xD2 => {
                let addr = self.read_word(bus);
                if !self.registers.f.carry {
                    self.jump_to(addr);
                }
                12
            }
            0xCA => {
                let addr = self.read_word(bus);
                if !self.registers.f.carry {
                    self.jump_to(addr);
                }
                12
            }
            /// JP (HL)
            0xE9 => {
                self.jump_to(self.registers.get_hl());
                4
            }
            /// JR n
            0x18 => {
                let offset = self.read_byte(bus) as i8;
                self.jump_relative(offset);
                8
            }
            /// JR cc, n
            0x20 => {
                let offset = self.read_byte(bus) as i8;
                if !self.registers.f.zero {
                    self.jump_relative(offset);
                }
                8
            }
            0x28 => {
                let offset = self.read_byte(bus) as i8;
                if self.registers.f.zero {
                    self.jump_relative(offset);
                }
                8
            }
            0x30 => {
                let offset = self.read_byte(bus) as i8;
                if !self.registers.f.carry {
                    self.jump_relative(offset);
                }
                8
            }
            0x38 => {
                let offset = self.read_byte(bus) as i8;
                if !self.registers.f.carry {
                    self.jump_relative(offset);
                }
                8
            }
            //
            // Calls
            //
            /// CALL nn
            0xCD => {
                let addr = self.read_word(bus);
                self.call(bus, addr);
                12
            }
            /// CALL cc, nn
            0xC4 => {
                let addr = self.read_word(bus);
                if !self.registers.f.zero {
                    self.call(bus, addr);
                }
                12
            }
            0xCC => {
                let addr = self.read_word(bus);
                if self.registers.f.zero {
                    self.call(bus, addr);
                }
                12
            }
            0xD4 => {
                let addr = self.read_word(bus);
                if !self.registers.f.carry {
                    self.call(bus, addr);
                }
                12
            }
            0xDC => {
                let addr = self.read_word(bus);
                if !self.registers.f.carry {
                    self.call(bus, addr);
                }
                12
            }
            //
            // Restarts
            //
            /// RST n
            0xC7 => {
                self.call(bus, 0x00);
                16
            }
            0xCF => {
                self.call(bus, 0x08);
                16
            }
            0xD7 => {
                self.call(bus, 0x10);
                16
            }
            0xDF => {
                self.call(bus, 0x18);
                16
            }
            0xE7 => {
                self.call(bus, 0x20);
                16
            }
            0xEF => {
                self.call(bus, 0x28);
                16
            }
            0xF7 => {
                self.call(bus, 0x30);
                16
            }
            0xFF => {
                self.call(bus, 0x38);
                16
            }
            //
            // Returns
            //
            /// RET
            0xC9 => {
                self.ret(bus);
                8
            }
            /// RET cc
            0xC0 => {
                if !self.registers.f.zero {
                    self.ret(bus);
                }
                8
            }
            0xC8 => {
                if self.registers.f.zero {
                    self.ret(bus);
                }
                8
            }
            0xD0 => {
                if !self.registers.f.carry {
                    self.ret(bus);
                }
                8
            }
            0xD8 => {
                if !self.registers.f.carry {
                    self.ret(bus);
                }
                8
            }
            /// RETI
            0xD9 => {
                self.ret(bus);
                // TODO: ENABLE INTERRUPTS
                8
            }
            // NOT FOUND!
            _ => {
                panic!(
                    "Instruction {:#X} not supported: {:#X}",
                    self.registers.pc - 1,
                    opcode
                );
            }
        };
        //println!("Cycles: {}", cycles);
        if self.registers.pc == 0x64 {
            println!("Waiting for LCD implementation!");
        }
        if self.registers.pc > 0x100 {
            println!("BOOT-ROM has exited!");
        }
    }

    fn step_cb(&mut self, bus: &mut Bus) -> i32 {
        let opcode = bus.ram_read_byte(self.registers.pc);
        //println!("CB instruction {:#X}: {:#X}", self.registers.pc, opcode);
        self.registers.pc += 1;

        let cycles = match opcode {
            // MISC
            /// SWAP n
            0x37 => {
                self.registers.a = self.swap(self.registers.a);
                8
            }
            0x30 => {
                self.registers.b = self.swap(self.registers.b);
                8
            }
            0x31 => {
                self.registers.c = self.swap(self.registers.c);
                8
            }
            0x32 => {
                self.registers.d = self.swap(self.registers.d);
                8
            }
            0x33 => {
                self.registers.e = self.swap(self.registers.e);
                8
            }
            0x34 => {
                self.registers.h = self.swap(self.registers.h);
                8
            }
            0x35 => {
                self.registers.l = self.swap(self.registers.l);
                8
            }
            0x36 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.swap(byte);
                self.write_ram_at_hl(bus, result);
                16
            }
            //
            // Shifts & Rotations
            //
            /// RLC n
            0x07 => {
                self.registers.a = self.rotate_circular(self.registers.a, true);
                8
            }
            0x00 => {
                self.registers.b = self.rotate_circular(self.registers.b, true);
                8
            }
            0x01 => {
                self.registers.c = self.rotate_circular(self.registers.c, true);
                8
            }
            0x02 => {
                self.registers.d = self.rotate_circular(self.registers.d, true);
                8
            }
            0x03 => {
                self.registers.e = self.rotate_circular(self.registers.e, true);
                8
            }
            0x04 => {
                self.registers.h = self.rotate_circular(self.registers.h, true);
                8
            }
            0x05 => {
                self.registers.l = self.rotate_circular(self.registers.l, true);
                8
            }
            0x06 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.rotate_circular(byte, true);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// RL n
            0x17 => {
                self.registers.a = self.rotate(self.registers.a, true);
                8
            }
            0x10 => {
                self.registers.b = self.rotate(self.registers.b, true);
                8
            }
            0x11 => {
                self.registers.c = self.rotate(self.registers.c, true);
                8
            }
            0x12 => {
                self.registers.d = self.rotate(self.registers.d, true);
                8
            }
            0x13 => {
                self.registers.e = self.rotate(self.registers.e, true);
                8
            }
            0x14 => {
                self.registers.h = self.rotate(self.registers.h, true);
                8
            }
            0x15 => {
                self.registers.l = self.rotate(self.registers.l, true);
                8
            }
            0x16 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.rotate(byte, true);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// RRC n
            0x0F => {
                self.registers.a = self.rotate_circular(self.registers.a, false);
                8
            }
            0x08 => {
                self.registers.b = self.rotate_circular(self.registers.b, false);
                8
            }
            0x09 => {
                self.registers.c = self.rotate_circular(self.registers.c, false);
                8
            }
            0x0A => {
                self.registers.d = self.rotate_circular(self.registers.d, false);
                8
            }
            0x0B => {
                self.registers.e = self.rotate_circular(self.registers.e, false);
                8
            }
            0x0C => {
                self.registers.h = self.rotate_circular(self.registers.h, false);
                8
            }
            0x0D => {
                self.registers.l = self.rotate_circular(self.registers.l, false);
                8
            }
            0x0E => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.rotate_circular(byte, false);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// RR n
            0x1F => {
                self.registers.a = self.rotate(self.registers.a, false);
                8
            }
            0x18 => {
                self.registers.b = self.rotate(self.registers.b, false);
                8
            }
            0x19 => {
                self.registers.c = self.rotate(self.registers.c, false);
                8
            }
            0x1A => {
                self.registers.d = self.rotate(self.registers.d, false);
                8
            }
            0x1B => {
                self.registers.e = self.rotate(self.registers.e, false);
                8
            }
            0x1C => {
                self.registers.h = self.rotate(self.registers.h, false);
                8
            }
            0x1D => {
                self.registers.l = self.rotate(self.registers.l, false);
                8
            }
            0x1E => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.rotate(byte, false);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// SLA n
            0x27 => {
                self.registers.a = self.shift(self.registers.a, true, false);
                8
            }
            0x20 => {
                self.registers.b = self.shift(self.registers.b, true, false);
                8
            }
            0x21 => {
                self.registers.c = self.shift(self.registers.c, true, false);
                8
            }
            0x22 => {
                self.registers.d = self.shift(self.registers.d, true, false);
                8
            }
            0x23 => {
                self.registers.e = self.shift(self.registers.e, true, false);
                8
            }
            0x24 => {
                self.registers.h = self.shift(self.registers.h, true, false);
                8
            }
            0x25 => {
                self.registers.l = self.shift(self.registers.l, true, false);
                8
            }
            0x26 => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.shift(byte, true, false);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// SRA n
            0x2F => {
                self.registers.a = self.shift(self.registers.a, false, true);
                8
            }
            0x28 => {
                self.registers.b = self.shift(self.registers.b, false, true);
                8
            }
            0x29 => {
                self.registers.c = self.shift(self.registers.c, false, true);
                8
            }
            0x2A => {
                self.registers.d = self.shift(self.registers.d, false, true);
                8
            }
            0x2B => {
                self.registers.e = self.shift(self.registers.e, false, true);
                8
            }
            0x2C => {
                self.registers.h = self.shift(self.registers.h, false, true);
                8
            }
            0x2D => {
                self.registers.l = self.shift(self.registers.l, false, true);
                8
            }
            0x2E => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.shift(byte, false, true);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// SRL n
            0x3F => {
                self.registers.a = self.shift(self.registers.a, false, false);
                8
            }
            0x38 => {
                self.registers.b = self.shift(self.registers.b, false, false);
                8
            }
            0x39 => {
                self.registers.c = self.shift(self.registers.c, false, false);
                8
            }
            0x3A => {
                self.registers.d = self.shift(self.registers.d, false, false);
                8
            }
            0x3B => {
                self.registers.e = self.shift(self.registers.e, false, false);
                8
            }
            0x3C => {
                self.registers.h = self.shift(self.registers.h, false, false);
                8
            }
            0x3D => {
                self.registers.l = self.shift(self.registers.l, false, false);
                8
            }
            0x3E => {
                let byte = self.read_ram_at_hl(bus);
                let result = self.shift(byte, false, false);
                self.write_ram_at_hl(bus, result);
                16
            }
            /// BIT b, r
            0x40 => {
                self.test_bit(self.registers.b, 0);
                8
            }
            0x41 => {
                self.test_bit(self.registers.c, 0);
                8
            }
            0x42 => {
                self.test_bit(self.registers.d, 0);
                8
            }
            0x43 => {
                self.test_bit(self.registers.e, 0);
                8
            }
            0x44 => {
                self.test_bit(self.registers.h, 0);
                8
            }
            0x45 => {
                self.test_bit(self.registers.l, 0);
                8
            }
            0x46 => {
                self.test_bit(self.read_ram_at_hl(bus), 0);
                12
            }
            0x47 => {
                self.test_bit(self.registers.a, 0);
                8
            }
            0x48 => {
                self.test_bit(self.registers.b, 1);
                8
            }
            0x49 => {
                self.test_bit(self.registers.c, 1);
                8
            }
            0x4A => {
                self.test_bit(self.registers.d, 1);
                8
            }
            0x4B => {
                self.test_bit(self.registers.e, 1);
                8
            }
            0x4C => {
                self.test_bit(self.registers.h, 1);
                8
            }
            0x4D => {
                self.test_bit(self.registers.l, 1);
                8
            }
            0x4E => {
                self.test_bit(self.read_ram_at_hl(bus), 1);
                12
            }
            0x4F => {
                self.test_bit(self.registers.a, 1);
                8
            }
            ///
            0x50 => {
                self.test_bit(self.registers.b, 2);
                8
            }
            0x51 => {
                self.test_bit(self.registers.c, 2);
                8
            }
            0x52 => {
                self.test_bit(self.registers.d, 2);
                8
            }
            0x53 => {
                self.test_bit(self.registers.e, 2);
                8
            }
            0x54 => {
                self.test_bit(self.registers.h, 2);
                8
            }
            0x55 => {
                self.test_bit(self.registers.l, 2);
                8
            }
            0x56 => {
                self.test_bit(self.read_ram_at_hl(bus), 2);
                12
            }
            0x57 => {
                self.test_bit(self.registers.a, 2);
                8
            }
            0x58 => {
                self.test_bit(self.registers.b, 3);
                8
            }
            0x59 => {
                self.test_bit(self.registers.c, 3);
                8
            }
            0x5A => {
                self.test_bit(self.registers.d, 3);
                8
            }
            0x5B => {
                self.test_bit(self.registers.e, 3);
                8
            }
            0x5C => {
                self.test_bit(self.registers.h, 3);
                8
            }
            0x5D => {
                self.test_bit(self.registers.l, 3);
                8
            }
            0x5E => {
                self.test_bit(self.read_ram_at_hl(bus), 3);
                12
            }
            0x5F => {
                self.test_bit(self.registers.a, 3);
                8
            }
            ///
            0x60 => {
                self.test_bit(self.registers.b, 4);
                8
            }
            0x61 => {
                self.test_bit(self.registers.c, 4);
                8
            }
            0x62 => {
                self.test_bit(self.registers.d, 4);
                8
            }
            0x63 => {
                self.test_bit(self.registers.e, 4);
                8
            }
            0x64 => {
                self.test_bit(self.registers.h, 4);
                8
            }
            0x65 => {
                self.test_bit(self.registers.l, 4);
                8
            }
            0x66 => {
                self.test_bit(self.read_ram_at_hl(bus), 4);
                12
            }
            0x67 => {
                self.test_bit(self.registers.a, 4);
                8
            }
            0x68 => {
                self.test_bit(self.registers.b, 5);
                8
            }
            0x69 => {
                self.test_bit(self.registers.c, 5);
                8
            }
            0x6A => {
                self.test_bit(self.registers.d, 5);
                8
            }
            0x6B => {
                self.test_bit(self.registers.e, 5);
                8
            }
            0x6C => {
                self.test_bit(self.registers.h, 5);
                8
            }
            0x6D => {
                self.test_bit(self.registers.l, 5);
                8
            }
            0x6E => {
                self.test_bit(self.read_ram_at_hl(bus), 5);
                12
            }
            0x6F => {
                self.test_bit(self.registers.a, 5);
                8
            }
            ///
            0x70 => {
                self.test_bit(self.registers.b, 6);
                8
            }
            0x71 => {
                self.test_bit(self.registers.c, 6);
                8
            }
            0x72 => {
                self.test_bit(self.registers.d, 6);
                8
            }
            0x73 => {
                self.test_bit(self.registers.e, 6);
                8
            }
            0x74 => {
                self.test_bit(self.registers.h, 6);
                8
            }
            0x75 => {
                self.test_bit(self.registers.l, 6);
                8
            }
            0x76 => {
                self.test_bit(self.read_ram_at_hl(bus), 6);
                12
            }
            0x77 => {
                self.test_bit(self.registers.a, 0);
                8
            }
            0x78 => {
                self.test_bit(self.registers.b, 7);
                8
            }
            0x79 => {
                self.test_bit(self.registers.c, 7);
                8
            }
            0x7A => {
                self.test_bit(self.registers.d, 7);
                8
            }
            0x7B => {
                self.test_bit(self.registers.e, 7);
                8
            }
            0x7C => {
                self.test_bit(self.registers.h, 7);
                8
            }
            0x7D => {
                self.test_bit(self.registers.l, 7);
                8
            }
            0x7E => {
                self.test_bit(self.read_ram_at_hl(bus), 7);
                12
            }
            0x7F => {
                self.test_bit(self.registers.a, 7);
                8
            }
            /// SET b, r
            0xC0 => {
                self.registers.b = self.set_bit(self.registers.b, 0);
                8
            }
            0xC1 => {
                self.registers.c = self.set_bit(self.registers.c, 0);
                8
            }
            0xC2 => {
                self.registers.d = self.set_bit(self.registers.d, 0);
                8
            }
            0xC3 => {
                self.registers.e = self.set_bit(self.registers.e, 0);
                8
            }
            0xC4 => {
                self.registers.h = self.set_bit(self.registers.h, 0);
                8
            }
            0xC5 => {
                self.registers.l = self.set_bit(self.registers.l, 0);
                8
            }
            0xC6 => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 0);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xC7 => {
                self.registers.a = self.set_bit(self.registers.a, 0);
                8
            }
            0xC8 => {
                self.registers.b = self.set_bit(self.registers.b, 1);
                8
            }
            0xC9 => {
                self.registers.c = self.set_bit(self.registers.c, 1);
                8
            }
            0xCA => {
                self.registers.d = self.set_bit(self.registers.d, 1);
                8
            }
            0xCB => {
                self.registers.e = self.set_bit(self.registers.e, 1);
                8
            }
            0xCC => {
                self.registers.h = self.set_bit(self.registers.h, 1);
                8
            }
            0xCD => {
                self.registers.l = self.set_bit(self.registers.l, 1);
                8
            }
            0xCE => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 1);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xCF => {
                self.registers.a = self.set_bit(self.registers.a, 1);
                8
            }
            ///
            0xD0 => {
                self.registers.b = self.set_bit(self.registers.b, 2);
                8
            }
            0xD1 => {
                self.registers.c = self.set_bit(self.registers.c, 2);
                8
            }
            0xD2 => {
                self.registers.d = self.set_bit(self.registers.d, 2);
                8
            }
            0xD3 => {
                self.registers.e = self.set_bit(self.registers.e, 2);
                8
            }
            0xD4 => {
                self.registers.h = self.set_bit(self.registers.h, 2);
                8
            }
            0xD5 => {
                self.registers.l = self.set_bit(self.registers.l, 2);
                8
            }
            0xD6 => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 2);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xD7 => {
                self.registers.a = self.set_bit(self.registers.a, 2);
                8
            }
            0xD8 => {
                self.registers.b = self.set_bit(self.registers.b, 3);
                8
            }
            0xD9 => {
                self.registers.c = self.set_bit(self.registers.c, 3);
                8
            }
            0xDA => {
                self.registers.d = self.set_bit(self.registers.d, 3);
                8
            }
            0xDB => {
                self.registers.e = self.set_bit(self.registers.e, 3);
                8
            }
            0xDC => {
                self.registers.h = self.set_bit(self.registers.h, 3);
                8
            }
            0xDD => {
                self.registers.l = self.set_bit(self.registers.l, 3);
                8
            }
            0xDE => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 3);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xDF => {
                self.registers.a = self.set_bit(self.registers.a, 3);
                8
            }
            ///
            0xE0 => {
                self.registers.b = self.set_bit(self.registers.b, 4);
                8
            }
            0xE1 => {
                self.registers.c = self.set_bit(self.registers.c, 4);
                8
            }
            0xE2 => {
                self.registers.d = self.set_bit(self.registers.d, 4);
                8
            }
            0xE3 => {
                self.registers.e = self.set_bit(self.registers.e, 4);
                8
            }
            0xE4 => {
                self.registers.h = self.set_bit(self.registers.h, 4);
                8
            }
            0xE5 => {
                self.registers.l = self.set_bit(self.registers.l, 4);
                8
            }
            0xE6 => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 4);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xE7 => {
                self.registers.a = self.set_bit(self.registers.a, 4);
                8
            }
            0xE8 => {
                self.registers.b = self.set_bit(self.registers.b, 5);
                8
            }
            0xE9 => {
                self.registers.c = self.set_bit(self.registers.c, 5);
                8
            }
            0xEA => {
                self.registers.d = self.set_bit(self.registers.d, 5);
                8
            }
            0xEB => {
                self.registers.e = self.set_bit(self.registers.e, 5);
                8
            }
            0xEC => {
                self.registers.h = self.set_bit(self.registers.h, 5);
                8
            }
            0xED => {
                self.registers.l = self.set_bit(self.registers.l, 5);
                8
            }
            0xEE => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 5);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xEF => {
                self.registers.a = self.set_bit(self.registers.a, 5);
                8
            }
            ///
            0xF0 => {
                self.registers.b = self.set_bit(self.registers.b, 6);
                8
            }
            0xF1 => {
                self.registers.c = self.set_bit(self.registers.c, 6);
                8
            }
            0xF2 => {
                self.registers.d = self.set_bit(self.registers.d, 6);
                8
            }
            0xF3 => {
                self.registers.e = self.set_bit(self.registers.e, 6);
                8
            }
            0xF4 => {
                self.registers.h = self.set_bit(self.registers.h, 6);
                8
            }
            0xF5 => {
                self.registers.l = self.set_bit(self.registers.l, 6);
                8
            }
            0xF6 => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 6);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xF7 => {
                self.registers.a = self.set_bit(self.registers.a, 6);
                8
            }
            0xF8 => {
                self.registers.b = self.set_bit(self.registers.b, 7);
                8
            }
            0xF9 => {
                self.registers.c = self.set_bit(self.registers.c, 7);
                8
            }
            0xFA => {
                self.registers.d = self.set_bit(self.registers.d, 7);
                8
            }
            0xFB => {
                self.registers.e = self.set_bit(self.registers.e, 7);
                8
            }
            0xFC => {
                self.registers.h = self.set_bit(self.registers.h, 7);
                8
            }
            0xFD => {
                self.registers.l = self.set_bit(self.registers.l, 7);
                8
            }
            0xFE => {
                let result = self.set_bit(self.read_ram_at_hl(bus), 7);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xFF => {
                self.registers.a = self.set_bit(self.registers.a, 7);
                8
            }
            /// RES b, r
            0x80 => {
                self.registers.b = self.reset_bit(self.registers.b, 0);
                8
            }
            0x81 => {
                self.registers.c = self.reset_bit(self.registers.c, 0);
                8
            }
            0x82 => {
                self.registers.d = self.reset_bit(self.registers.d, 0);
                8
            }
            0x83 => {
                self.registers.e = self.reset_bit(self.registers.e, 0);
                8
            }
            0x84 => {
                self.registers.h = self.reset_bit(self.registers.h, 0);
                8
            }
            0x85 => {
                self.registers.l = self.reset_bit(self.registers.l, 0);
                8
            }
            0x86 => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 0);
                self.write_ram_at_hl(bus, result);
                16
            }
            0x87 => {
                self.registers.a = self.reset_bit(self.registers.a, 0);
                8
            }
            0x88 => {
                self.registers.b = self.reset_bit(self.registers.b, 1);
                8
            }
            0x89 => {
                self.registers.c = self.reset_bit(self.registers.c, 1);
                8
            }
            0x8A => {
                self.registers.d = self.reset_bit(self.registers.d, 1);
                8
            }
            0x8B => {
                self.registers.e = self.reset_bit(self.registers.e, 1);
                8
            }
            0x8C => {
                self.registers.h = self.reset_bit(self.registers.h, 1);
                8
            }
            0x8D => {
                self.registers.l = self.reset_bit(self.registers.l, 1);
                8
            }
            0x8E => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 1);
                self.write_ram_at_hl(bus, result);
                16
            }
            0x8F => {
                self.registers.a = self.reset_bit(self.registers.a, 1);
                8
            }
            ///
            0x90 => {
                self.registers.b = self.reset_bit(self.registers.b, 2);
                8
            }
            0x91 => {
                self.registers.c = self.reset_bit(self.registers.c, 2);
                8
            }
            0x92 => {
                self.registers.d = self.reset_bit(self.registers.d, 2);
                8
            }
            0x93 => {
                self.registers.e = self.reset_bit(self.registers.e, 2);
                8
            }
            0x94 => {
                self.registers.h = self.reset_bit(self.registers.h, 2);
                8
            }
            0x95 => {
                self.registers.l = self.reset_bit(self.registers.l, 2);
                8
            }
            0x96 => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 2);
                self.write_ram_at_hl(bus, result);
                16
            }
            0x97 => {
                self.registers.a = self.reset_bit(self.registers.a, 2);
                8
            }
            0x98 => {
                self.registers.b = self.reset_bit(self.registers.b, 3);
                8
            }
            0x99 => {
                self.registers.c = self.reset_bit(self.registers.c, 3);
                8
            }
            0x9A => {
                self.registers.d = self.reset_bit(self.registers.d, 3);
                8
            }
            0x9B => {
                self.registers.e = self.reset_bit(self.registers.e, 3);
                8
            }
            0x9C => {
                self.registers.h = self.reset_bit(self.registers.h, 3);
                8
            }
            0x9D => {
                self.registers.l = self.reset_bit(self.registers.l, 3);
                8
            }
            0x9E => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 3);
                self.write_ram_at_hl(bus, result);
                16
            }
            0x9F => {
                self.registers.a = self.reset_bit(self.registers.a, 3);
                8
            }
            ///
            0xA0 => {
                self.registers.b = self.reset_bit(self.registers.b, 4);
                8
            }
            0xA1 => {
                self.registers.c = self.reset_bit(self.registers.c, 4);
                8
            }
            0xA2 => {
                self.registers.d = self.reset_bit(self.registers.d, 4);
                8
            }
            0xA3 => {
                self.registers.e = self.reset_bit(self.registers.e, 4);
                8
            }
            0xA4 => {
                self.registers.h = self.reset_bit(self.registers.h, 4);
                8
            }
            0xA5 => {
                self.registers.l = self.reset_bit(self.registers.l, 4);
                8
            }
            0xA6 => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 4);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xA7 => {
                self.registers.a = self.reset_bit(self.registers.a, 4);
                8
            }
            0xA8 => {
                self.registers.b = self.reset_bit(self.registers.b, 5);
                8
            }
            0xA9 => {
                self.registers.c = self.reset_bit(self.registers.c, 5);
                8
            }
            0xAA => {
                self.registers.d = self.reset_bit(self.registers.d, 5);
                8
            }
            0xAB => {
                self.registers.e = self.reset_bit(self.registers.e, 5);
                8
            }
            0xAC => {
                self.registers.h = self.reset_bit(self.registers.h, 5);
                8
            }
            0xAD => {
                self.registers.l = self.reset_bit(self.registers.l, 5);
                8
            }
            0xAE => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 5);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xAF => {
                self.registers.a = self.reset_bit(self.registers.a, 5);
                8
            }
            ///
            0xB0 => {
                self.registers.b = self.reset_bit(self.registers.b, 6);
                8
            }
            0xB1 => {
                self.registers.c = self.reset_bit(self.registers.c, 6);
                8
            }
            0xB2 => {
                self.registers.d = self.reset_bit(self.registers.d, 6);
                8
            }
            0xB3 => {
                self.registers.e = self.reset_bit(self.registers.e, 6);
                8
            }
            0xB4 => {
                self.registers.h = self.reset_bit(self.registers.h, 6);
                8
            }
            0xB5 => {
                self.registers.l = self.reset_bit(self.registers.l, 6);
                8
            }
            0xB6 => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 6);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xB7 => {
                self.registers.a = self.reset_bit(self.registers.a, 6);
                8
            }
            0xB8 => {
                self.registers.b = self.reset_bit(self.registers.b, 7);
                8
            }
            0xB9 => {
                self.registers.c = self.reset_bit(self.registers.c, 7);
                8
            }
            0xBA => {
                self.registers.d = self.reset_bit(self.registers.d, 7);
                8
            }
            0xBB => {
                self.registers.e = self.reset_bit(self.registers.e, 7);
                8
            }
            0xBC => {
                self.registers.h = self.reset_bit(self.registers.h, 7);
                8
            }
            0xBD => {
                self.registers.l = self.reset_bit(self.registers.l, 7);
                8
            }
            0xBE => {
                let result = self.reset_bit(self.read_ram_at_hl(bus), 7);
                self.write_ram_at_hl(bus, result);
                16
            }
            0xBF => {
                self.registers.a = self.reset_bit(self.registers.a, 7);
                8
            }
        };

        cycles
    }

    // Helper Methods
    fn form_16bit(a: u8, b: u8) -> u16 {
        ((a as u16) << 8) | (b as u16)
    }
    fn read_byte(&mut self, bus: &Bus) -> u8 {
        let imm1 = bus.ram_read_byte(self.registers.pc);
        self.registers.pc += 1; // Consumed one byte

        imm1
    }
    fn read_word(&mut self, bus: &Bus) -> u16 {
        let imm1 = bus.ram_read_byte(self.registers.pc);
        let imm2 = bus.ram_read_byte(self.registers.pc + 1);
        let addr: u16 = CPU::form_16bit(imm2, imm1); // Reverse order due to Big Endian
        self.registers.pc += 2; // Consumed two bytes

        addr
    }
    fn read_ram_at_hl(&self, bus: &Bus) -> u8 {
        bus.ram_read_byte(self.registers.get_hl())
    }
    fn write_ram_at_hl(&self, bus: &mut Bus, byte: u8) {
        bus.ram_write_byte(self.registers.get_hl(), byte);
    }

    // STACK
    fn stack_push(&mut self, bus: &mut Bus, word: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        bus.ram_write_word(self.registers.sp, word);
    }

    fn stack_pop(&mut self, bus: &mut Bus) -> u16 {
        let word = bus.ram_read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);

        word
    }

    // ALU
    /// ALU 8-Bit
    fn alu_add(&mut self, b: u8, carry: bool) {
        let c = if carry && self.registers.f.carry {
            1
        } else {
            0
        };
        let a = self.registers.a;
        let result = a.wrapping_add(b).wrapping_add(c);

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(
            Flag::H,
            ((a & 0xF) as u16) + ((b & 0xF) as u16) + ((c & 0xF) as u16) > 0xF,
        );
        self.registers
            .f
            .flag(Flag::C, (a as u16) + (b as u16) + (c as u16) > 0xFF);

        self.registers.a = result;
    }
    fn alu_sub(&mut self, b: u8, carry: bool) {
        let c = if carry && self.registers.f.carry {
            1
        } else {
            0
        };
        let a = self.registers.a;
        let result = a.wrapping_sub((b.wrapping_add(c)));

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, true);
        self.registers.f.flag(Flag::H, (a & 0xF) < (b & 0xF) + c);
        self.registers
            .f
            .flag(Flag::C, (a as u16) < (b as u16) + (c as u16));

        self.registers.a = result;
    }
    fn alu_and(&mut self, b: u8) {
        let result = self.registers.a & b;

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, true);
        self.registers.f.flag(Flag::C, false);

        self.registers.a = result;
    }
    fn alu_or(&mut self, b: u8) {
        let result = self.registers.a | b;

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, false);

        self.registers.a = result;
    }
    fn alu_xor(&mut self, b: u8) {
        let result = self.registers.a ^ b;

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, false);

        self.registers.a = result;
    }
    fn alu_cp(&mut self, b: u8) {
        // Compare A with n
        let a = self.registers.a;
        let result = a.wrapping_sub(b);

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, true);
        self.registers.f.flag(Flag::H, (a & 0x7) > (b & 0x7));
        self.registers.f.flag(Flag::C, a < b);

        // Throwaway result
    }
    fn inc(&mut self, a: u8) -> u8 {
        let result = a.wrapping_add(1);

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, (a & 0xF) + 1 > 0xF);

        result
    }
    fn dec(&mut self, a: u8) -> u8 {
        let result = a.wrapping_sub(1);

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, true);
        self.registers.f.flag(Flag::H, (a & 0x0F) == 0);

        result
    }
    /// ALU 16-Bit
    fn alu_add_16_imm(&mut self, a: u16, b: u8) -> u16 {
        let result = a.wrapping_add(b as u16);

        self.registers.f.flag(Flag::Z, false);
        self.registers.f.flag(Flag::N, false);
        self.registers
            .f
            .flag(Flag::H, (a & 0xF) + ((b as u16) & 0xF) > 0xF);
        self.registers
            .f
            .flag(Flag::C, (a & 0x00FF) + ((b as u16) & 0x00FF) > 0x00FF);

        result
    }
    fn alu_add_16(&mut self, b: u16) {
        let a = self.registers.get_hl();
        let result = a.wrapping_add(b);

        self.registers.f.flag(Flag::N, false);
        self.registers
            .f
            .flag(Flag::H, ((a & 0xFFF) as u32) + ((b & 0x7FF) as u32) > 0x7FF);
        self.registers
            .f
            .flag(Flag::C, (a as u32) + (b as u32) > 0xFFFF);

        self.registers.set_hl(result);
    }

    // MISC
    fn swap(&mut self, a: u8) -> u8 {
        let upper = a & 0xF0;
        let lower = a & 0x0F;
        let result = (lower << 4) | (upper >> 4);

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, false);

        result
    }

    // Rotates & Shifts
    fn rotate_circular(&mut self, a: u8, left: bool) -> u8 {
        let lost_bit = if left { a & 0b10000000 } else { a & 0b00000001 };
        let mut result = if left { a << 1 } else { a >> 1 };
        result |= if left { lost_bit >> 7 } else { lost_bit << 7 };

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, lost_bit != 0);

        result
    }

    fn rotate(&mut self, a: u8, left: bool) -> u8 {
        let lost_bit = if left { a & 0b1000000 } else { a & 0b00000001 };
        let mut result = if left { a << 1 } else { a >> 1 };
        result |= if self.registers.f.carry {
            if left {
                1
            } else {
                0b10000000
            }
        } else {
            0
        };

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, lost_bit != 0);

        result
    }

    fn shift(&mut self, a: u8, left: bool, retain_msb: bool) -> u8 {
        let outter_bit = if left { a & 0b10000000 } else { a & 0b00000001 };
        let mut result = if left { a << 1 } else { a >> 1 };

        if !left && retain_msb {
            result |= outter_bit;
        }

        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, false);
        self.registers.f.flag(Flag::C, outter_bit != 0);

        result
    }

    fn test_bit(&mut self, a: u8, b: u8) {
        let mask = 0b1 << b;
        let bit = a & mask;

        self.registers.f.flag(Flag::Z, bit == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, true);
    }

    fn set_bit(&mut self, a: u8, b: u8) -> u8 {
        let mask = 0b1 << b;
        let result = a | mask;

        result
    }

    fn reset_bit(&mut self, a: u8, b: u8) -> u8 {
        let mask = 0b1 << b;
        let result = a & !mask;

        result
    }

    // Jumps
    fn jump_to(&mut self, word: u16) {
        self.registers.pc = word;
    }

    fn jump_relative(&mut self, byte: i8) {
        self.registers.pc = ((self.registers.pc as u32 as i32) + (byte as i32)) as u16;
    }

    // Calls
    fn call(&mut self, bus: &mut Bus, word: u16) {
        self.stack_push(bus, self.registers.pc);
        self.jump_to(word);
    }

    // Restarts
    fn rst(&mut self, word: u16) {}

    // Returns
    fn ret(&mut self, bus: &mut Bus) {
        let addr = self.stack_pop(bus);
        self.jump_to(addr);
    }
}

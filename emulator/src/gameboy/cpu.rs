use super::register;
use super::Bus;
use super::Flag;
use super::Registers;

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
        println!("instruction {:#X}: {:#X}", self.registers.pc, opcode);
        self.registers.pc += 1;

        let hex1 = (opcode & 0xF0) << 1;
        let hex2 = opcode & 0x0F;

        match opcode {
            // 16-bit load instructions (LD n, nn)
            0x01 | 0x11 | 0x21 | 0x31 => {
                self.ld_16_n_nn(bus, opcode);
            }
            // 8-bit XOR instructions
            0xAF | 0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAE | 0xEE => {
                self.xor_8_n(bus, opcode);
            }
            0x22 => self.ld_into_hl(bus, true),  // LD (HL+)
            0x32 => self.ld_into_hl(bus, false), // LD (HL-)
            0xCB => self.step_cb(bus),           // CB prefix
            0x20 | 0x28 | 0x30 | 0x38 => self.jr_cc_n(bus, opcode), // Jump conditional
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E => self.ld_8_nn_n(bus, opcode), // Load nn, n
            0x7F | 0x78 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x0A | 0x1A | 0x7E | 0xFA | 0x3E => {
                self.ld_8_a_n(bus, opcode)
            }
            0xE2 => self.ld_c_a(bus),
            0xC9 => self.stack_return(bus),
            0xC0 | 0xC8 | 0xD0 | 0xD8 => self.ret_cc(bus, opcode),
            0x3C | 0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 => self.inc_n(bus, opcode),
            0x7F | 0x47 | 0x4F | 0x57 | 0x5F | 0x67 | 0x6F | 0x02 | 0x12 | 0x77 | 0xEA => {
                self.ld_n_a(bus, opcode)
            }
            0xE0 => self.ldh_n_a(bus),
            0xF0 => self.ldh_a_n(bus),
            0xCD => self.call_nn(bus),
            0xF5 | 0xC5 | 0xD5 | 0xE5 => self.push_nn(bus, opcode),
            _ => panic!(
                "Instruction {:#X} not supported: {:#X}",
                self.registers.pc - 1,
                opcode
            ),
        }

        self.registers.dump();
    }

    fn step_cb(&mut self, bus: &mut Bus) {
        let opcode = bus.ram_read_byte(self.registers.pc);
        println!("CB instruction {:#X}: {:#X}", self.registers.pc, opcode);
        self.registers.pc += 1;

        match opcode {
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x46 | 0x47 | 0x48 | 0x49 | 0x4A | 0x4B
            | 0x4C | 0x4D | 0x4E | 0x4F | 0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57
            | 0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5E | 0x5F | 0x60 | 0x61 | 0x62 | 0x63
            | 0x64 | 0x65 | 0x66 | 0x67 | 0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6E | 0x6F
            | 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 | 0x79 | 0x7A | 0x7B
            | 0x7C | 0x7D | 0x7E | 0x7F => self.bit(bus, opcode),
            _ => panic!(
                "Instruction {:#X} not supported: {:#X}",
                self.registers.pc - 1,
                opcode
            ),
        }
    }

    // Helper Methods
    fn form_16bit(a: u8, b: u8) -> u16 {
        ((a as u16) << 8) | (b as u16)
    }
    fn read_8bits(&mut self, bus: &Bus) -> u8 {
        let imm1 = bus.ram_read_byte(self.registers.pc);
        self.registers.pc += 1; // Consumed one byte

        imm1
    }
    fn read_16bits(&mut self, bus: &Bus) -> u16 {
        let imm1 = bus.ram_read_byte(self.registers.pc);
        let imm2 = bus.ram_read_byte(self.registers.pc + 1);
        let addr: u16 = CPU::form_16bit(imm2, imm1); // Reverse order due to Big Endian
        self.registers.pc += 2; // Consumed two bytes

        addr
    }

    fn read_ram_hl(&self, bus: &Bus) -> u8 {
        bus.ram_read_byte(self.registers.get_hl())
    }
    fn write_ram_hl(&self, bus: &mut Bus, byte: u8) {
        bus.ram_write_byte(self.registers.get_hl(), byte);
    }

    // Instruction Set
    /// 8-bit load instructions
    fn ld_8_nn_n(&mut self, bus: &mut Bus, opcode: u8) {
        // LD reg, value
        // Reverse order for Big Endian
        let n = self.read_8bits(bus);

        match opcode {
            0x06 => {
                self.registers.b = n;
            }
            0x0E => {
                self.registers.c = n;
            }
            0x16 => {
                self.registers.d = n;
            }
            0x1E => {
                self.registers.e = n;
            }
            0x26 => {
                self.registers.h = n;
            }
            0x2E => {
                self.registers.l = n;
            }
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }
    }

    fn ld_8_a_n(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0x7F => { /* Nothing */ }
            0x78 => self.registers.a = self.registers.b,
            0x79 => self.registers.a = self.registers.c,
            0x7A => self.registers.a = self.registers.d,
            0x7B => self.registers.a = self.registers.e,
            0x7C => self.registers.a = self.registers.h,
            0x7D => self.registers.a = self.registers.l,
            0x0A => self.registers.a = bus.ram_read_byte(self.registers.get_bc()),
            0x1A => self.registers.a = bus.ram_read_byte(self.registers.get_de()),
            0x7E => self.registers.a = bus.ram_read_byte(self.registers.get_hl()),
            0xFA => self.registers.a = bus.ram_read_byte(self.read_16bits(bus)),
            0x3E => self.registers.a = self.read_8bits(bus),
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }
    }

    fn ld_n_a(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0x7F => self.registers.a = self.registers.a,
            0x47 => self.registers.b = self.registers.a,
            0x4F => self.registers.c = self.registers.a,
            0x57 => self.registers.d = self.registers.a,
            0x5F => self.registers.e = self.registers.a,
            0x67 => self.registers.h = self.registers.a,
            0x6F => self.registers.l = self.registers.a,
            0x02 => bus.ram_write_byte(self.registers.get_bc(), self.registers.a),
            0x12 => bus.ram_write_byte(self.registers.get_de(), self.registers.a),
            0x77 => bus.ram_write_byte(self.registers.get_hl(), self.registers.a),
            0xEA => {
                let addr = self.read_16bits(bus);
                bus.ram_write_byte(addr, self.registers.a);
            }
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }
    }

    fn ld_c_a(&mut self, bus: &mut Bus) {
        bus.ram_write_byte(0xFF00 + (self.registers.c as u16), self.registers.a)
        // 0xFF | C
    }

    fn ldh_n_a(&mut self, bus: &mut Bus) {
        let byte = self.read_8bits(bus);
        let addr = 0xFF00 + (byte as u16);
        bus.ram_write_byte(addr, self.registers.a);
    }

    fn ldh_a_n(&mut self, bus: &mut Bus) {
        let byte = self.read_8bits(bus);
        let addr = 0xFF00 + (byte as u16);
        self.registers.a = bus.ram_read_byte(addr);
    }

    fn push_nn(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0xAF => self.stack_push(bus, self.registers.get_af()),
            0xC5 => self.stack_push(bus, self.registers.get_bc()),
            0xD5 => self.stack_push(bus, self.registers.get_de()),
            0xE5 => self.stack_push(bus, self.registers.get_hl()),
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }
    }

    // 8-bit ALU

    fn xor_8_n(&mut self, bus: &mut Bus, opcode: u8) {
        // Modifies FlagsRegister, result of xor => Register A
        let operand: u8;

        match opcode {
            0xAF => operand = self.registers.a,
            0xA8 => operand = self.registers.b,
            0xA9 => operand = self.registers.c,
            0xAA => operand = self.registers.d,
            0xAB => operand = self.registers.e,
            0xAC => operand = self.registers.h,
            0xAD => operand = self.registers.l,
            0xAE => operand = self.read_ram_hl(bus), // 16-bit register destination
            0xEE => operand = self.read_8bits(bus),
            _ => {
                panic!("Unsupported XOR opcode: {:#X}", opcode);
            }
        }

        let result = self.registers.a & operand;
        self.registers.f.set(result == 0, false, false, false);
        self.registers.a = result;
        //self.registers.f.flag(Flag::Z, result == 0);
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let result = a.wrapping_add(1);
        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, (a & 0xF) == 0xF);

        result
    }

    fn inc_n(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0x3C => self.registers.a = self.alu_inc(self.registers.a),
            0x04 => self.registers.b = self.alu_inc(self.registers.b),
            0x0C => self.registers.c = self.alu_inc(self.registers.c),
            0x14 => self.registers.d = self.alu_inc(self.registers.d),
            0x1C => self.registers.e = self.alu_inc(self.registers.e),
            0x24 => self.registers.h = self.alu_inc(self.registers.h),
            0x2C => self.registers.l = self.alu_inc(self.registers.l),
            0x34 => {
                let mut byte: u8 = self.read_ram_hl(bus);
                byte = self.alu_inc(byte);
                self.write_ram_hl(bus, byte);
            }
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }
    }

    /// 16-bit load instructions
    fn ld_16_n_nn(&mut self, bus: &Bus, opcode: u8) {
        let value: u16 = self.read_16bits(bus);
        println!("\tvalue: {:#X}", value);

        match opcode {
            0x01 => self.registers.set_bc(value),
            0x11 => self.registers.set_de(value),
            0x21 => self.registers.set_hl(value),
            0x31 => self.registers.sp = value,
            _ => {
                panic!("Unsupported nn for {:#X}", opcode)
            }
        }
    }

    fn ld_from_hl(&mut self, bus: &mut Bus, inc: bool) {
        self.registers.a = self.read_ram_hl(bus);
        self._modify_hl(inc);
    }
    fn ld_into_hl(&mut self, bus: &mut Bus, inc: bool) {
        self.write_ram_hl(bus, self.registers.a);
        self._modify_hl(inc);
    }

    fn _modify_hl(&mut self, inc: bool) {
        let mut hl = self.registers.get_hl();
        if inc {
            hl = hl + 1;
        } else {
            hl = hl - 1;
        }

        self.registers.set_hl(hl);
    }

    // Bit Opcodes
    fn bit(&mut self, bus: &mut Bus, opcode: u8) {
        let hex1: u8 = (opcode & 0xF0) >> 4;

        // operand
        let operand: u8;
        match hex1 {
            0x7 | 0xF => operand = self.registers.a,
            0x0 | 0x8 => operand = self.registers.b,
            0x1 | 0x9 => operand = self.registers.c,
            0x2 | 0xA => operand = self.registers.d,
            0x3 | 0xB => operand = self.registers.e,
            0x4 | 0xC => operand = self.registers.h,
            0x5 | 0xD => operand = self.registers.l,
            0x6 | 0xE => operand = self.read_ram_hl(bus), // 16-bit register destination
            _ => operand = self.registers.a,              // This fall-through should never occur
        }

        // b
        let b: u8;
        if 0x40 <= operand && operand <= 0x47 {
            b = 0;
        } else if 0x48 <= operand && operand <= 0x4F {
            b = 1;
        } else if 0x50 <= operand && operand <= 0x57 {
            b = 2;
        } else if 0x58 <= operand && operand <= 0x5F {
            b = 3;
        } else if 0x60 <= operand && operand <= 0x67 {
            b = 4;
        } else if 0x68 <= operand && operand <= 0x6F {
            b = 5;
        } else if 0x70 <= operand && operand <= 0x7F {
            b = 6;
        } else {
            b = 7;
        }

        println!("\tb: {:#X}", b);
        let mask = 1 << b;
        let result = operand & mask;
        self.registers.f.flag(Flag::Z, result == 0);
        self.registers.f.flag(Flag::N, false);
        self.registers.f.flag(Flag::H, true);
    }

    // Jump instructions
    fn jr_cc_n(&mut self, bus: &mut Bus, opcode: u8) {
        let n = self.read_8bits(bus);
        let cc = self.read_8bits(bus);
        println!("\tn: {:#X}", n);
        println!("\tcc: {:#X}", cc);

        let mut jump = false;

        match opcode {
            0x20 => {
                if !self.registers.f.zero {
                    jump = true
                }
            }
            0x28 => {
                if self.registers.f.zero {
                    jump = true
                }
            }
            0x30 => {
                if !self.registers.f.carry {
                    jump = true
                }
            }
            0x38 => {
                if self.registers.f.carry {
                    jump = true
                }
            }
            _ => {}
        }

        if jump {
            self.registers.pc = self.registers.pc + (n as u16); // TODO: perform safe adding?
        }
    }

    // Returns
    fn stack_pop(&mut self, bus: &mut Bus) -> u16 {
        let word = bus.ram_read_word(self.registers.sp);
        self.registers.sp += 2;

        word
    }

    fn stack_push(&mut self, bus: &mut Bus, word: u16) {
        bus.ram_write_word(self.registers.sp, word);
        self.registers.sp -= 2;
    }

    fn stack_return(&mut self, bus: &mut Bus) {
        //let lsb = self.stack_pop(bus);
        //let msb = self.stack_pop(bus);
        //let addr = ((msb as u16) >> 4) | (lsb as u16);
        let addr = self.stack_pop(bus);
        self.registers.pc = addr;
    }

    pub fn ret_cc(&mut self, bus: &mut Bus, opcode: u8) {
        let mut jump = false;

        match opcode {
            0xC0 => {
                if !self.registers.f.zero {
                    jump = true
                }
            }
            0xC8 => {
                if self.registers.f.zero {
                    jump = true
                }
            }
            0xD0 => {
                if !self.registers.f.carry {
                    jump = true
                }
            }
            0xD8 => {
                if self.registers.f.carry {
                    jump = true
                }
            }
            _ => {
                panic!("Not implemented: {:#X}", opcode);
            }
        }

        if jump {
            self.stack_return(bus);
        }
    }

    // Calls
    fn call_nn(&mut self, bus: &mut Bus) {
        self.stack_push(bus, self.registers.pc);

        let word = self.read_16bits(bus);
        self.registers.pc = word;
    }

    // Rotates & Shifts
    fn rl_n(&mut self, bus: &mut Bus) {}
}

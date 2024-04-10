use super::Bus;
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
        let instruction_byte = bus.ram_read_byte(self.registers.pc);
        println!(
            "instruction {:#X}: {:#X}",
            self.registers.pc, instruction_byte
        );

        let hex1 = (instruction_byte & 0xF0) << 1;
        let hex2 = instruction_byte & 0x0F;

        match (instruction_byte) {
            (0xCB) => {
                // PREFIX - 2nd table lookup
            }
            _ => panic!(
                "Instruction {:#X} not supported: {:#X}",
                self.registers.pc, instruction_byte
            ),
        }

        if instruction_byte == 0xCB {
            self.registers.pc += 1;
            println!(
                "\tPrefix bit found! Next byte: {:#X}",
                bus.ram_read_byte(self.registers.pc)
            );
        }

        self.registers.pc += 1;
    }
}

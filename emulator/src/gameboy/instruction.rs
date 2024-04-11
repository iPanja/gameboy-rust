pub enum Instruction {
    r8,  // Any of the 8-bit registers
    r16, // Any of the 16-bit registers (two 8-bit registers combined)
    r16stk,
    r16mem,
    cond,
    b3,
    tgt3,
    imm8,
    imm16,
}

pub enum InstructionModifier {}

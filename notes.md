# CPU Instruction Set

## General Purpose Registers

- Registers A-L (8 bits)
  - Register A - Accumulator, used to perform mathematical operations; argument for calls
  - Register F - Flag (i.e. this value is used for conditional jump operations)
    - (7 6 5 4 3 2 1 0)
    - (Z N H C - - - -)
    - (Zero flag, sub flag, half carry flag, carry flag)
  - Register H, L - often combined to form a 16-bit memory address (for indirect memory access)
- PC, SP (16 bits)

### F Register

Used to store the flag bits for certain operations.

| 7         | 6        | 5               | 4          | 3   | 2   | 1   | 0   |
| --------- | -------- | --------------- | ---------- | --- | --- | --- | --- |
| Zero flag | Sub flag | Half carry flag | Carry flag | 0   | 0   | 0   | 0   |

**Example**: 250_10 + 200_10 = 450_10 (overflow for 8 bits)

- 450 % 256 = 194 => result of operation
- 450 / 256 = 1 => carry bit of F (bit 4)

Other information:

- The carry bit (bit 4) can be used to check a jump condition
- The carry flag can be used to store a bit from an overflowed operation and by rotate functions
  - In rotate functions, this bit is used as an extra bit or duplicate of the rotated bit
    - Ex: Rotate Right Circular (RRC) operation. 1st bit => carry and to the 8-th bit. The other 7 bits are shifted to the right.

## Example Instructions

11000011 (C3)

- 1100 (C)
- 0011 (3)

C3 => JP instruction

## Architecture

- CPU
- RAM (Random Access Memory)
- Cartridge
  - ROM (Read Only Memory) - 32KB
- I/O
  - The screen
  - Sound hardware
  - Gamepad

### CPU

The Game Boy's CPU is a Sharp LR35902. It is very simliar to the Intel 8080 and the Zilog Z80.

Aspects:

- No pipelining
- Single processor
- Sprites are offloaded from the CPU
  - The CPU still has to move and load background elements from the memory to the designated video memory

## Opcodes

| _8-bit_  | 8-bit  | _8-bit_     | _8-bit_     |
| -------- | ------ | ----------- | ----------- |
| _prefix_ | opcode | _immediate_ | _immediate_ |

- 8-bit opcode => 256 opcodes
- An additional prefix bit (_opcode prefix_) => 256 \* 2 = 512 opcodes
- The length of the immediate is predetermined by the opcode
  - Can be either 8- or 16-bit
- Typically, the CPU only supports unsignd integers
  - Exception: Relative jumps and two Stack Pointer operations
- Big Endian (LSB <-> MSB)

## ALU

### Operations

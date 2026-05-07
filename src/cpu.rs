#![allow(dead_code)]
/// TermCraft Redstone CPU — 4-bit virtual processor
/// 
/// Architecture:
/// - 4-bit data bus
/// - 8 registers (R0-R7), 4-bit each
/// - 16 bytes of RAM (4-bit address space)
/// - Program counter (4-bit)
/// - Flags: Zero (Z), Carry (C)
///
/// Instruction Set (8-bit opcode: 4-bit instruction + 4-bit operand):
/// - 0x0N: NOP
/// - 0x1N: LOAD R[N] <- immediate (next byte)
/// - 0x2N: MOV R[N] <- R[operand]
/// - 0x3N: ADD R[N] += R[operand]
/// - 0x4N: SUB R[N] -= R[operand]
/// - 0x5N: AND R[N] &= R[operand]
/// - 0x6N: OR  R[N] |= R[operand]
/// - 0x7N: XOR R[N] ^= R[operand]
/// - 0x8N: JMP to address N
/// - 0x9N: JZ (jump if zero flag)
/// - 0xAN: JNZ (jump if not zero)
/// - 0xBN: STORE RAM[R[operand]] <- R[N]
/// - 0xCN: LOAD_R R[N] <- RAM[R[operand]]
/// - 0xDN: OUT (output R[N] to redstone signal)
/// - 0xEN: IN (read redstone signal into R[N])
/// - 0xF0: HALT

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedstoneCPU {
    pub registers: [u8; 8],    // R0-R7, 4-bit values (stored as u8, masked to 0xF)
    pub ram: [u8; 16],         // 16 bytes of RAM
    pub pc: u8,                // Program counter (4-bit)
    pub flags_z: bool,         // Zero flag
    pub flags_c: bool,         // Carry flag
    pub program: Vec<u8>,      // Program memory
    pub output: u8,            // Output register (maps to redstone signal)
    pub running: bool,
    pub cycle_count: u64,
}

impl RedstoneCPU {
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            ram: [0; 16],
            pc: 0,
            flags_z: false,
            flags_c: false,
            program: Vec::new(),
            output: 0,
            running: false,
            cycle_count: 0,
        }
    }

    /// Load a program into the CPU
    pub fn load_program(&mut self, program: &[u8]) {
        self.program = program.to_vec();
        self.pc = 0;
        self.running = true;
        self.cycle_count = 0;
    }

    /// Execute one clock cycle
    pub fn tick(&mut self) {
        if !self.running || self.pc as usize >= self.program.len() {
            self.running = false;
            return;
        }

        let opcode = self.program[self.pc as usize];
        let instruction = (opcode >> 4) & 0xF;
        let operand = opcode & 0xF;

        self.pc = (self.pc + 1) & 0xF; // wrap at 16

        match instruction {
            0x0 => {} // NOP

            0x1 => {
                // LOAD R[N] <- immediate
                let immediate = if (self.pc as usize) < self.program.len() {
                    self.program[self.pc as usize] & 0xF
                } else {
                    0
                };
                self.pc = (self.pc + 1) & 0xF;
                self.registers[operand as usize] = immediate;
            }

            0x2 => {
                // MOV R[N] <- R[(N+1) % 8]
                self.registers[operand as usize] = self.registers[((operand + 1) % 8) as usize];
            }

            0x3 => {
                // ADD R[N] += R[operand]
                let a = self.registers[operand as usize];
                let b = self.registers[((operand + 1) % 8) as usize];
                let result = a.wrapping_add(b);
                self.flags_c = a as u16 + b as u16 > 0xF;
                self.flags_z = (result & 0xF) == 0;
                self.registers[operand as usize] = result & 0xF;
            }

            0x4 => {
                // SUB R[N] -= R[operand]
                let a = self.registers[operand as usize];
                let b = self.registers[((operand + 1) % 8) as usize];
                let result = a.wrapping_sub(b);
                self.flags_c = b > a;
                self.flags_z = (result & 0xF) == 0;
                self.registers[operand as usize] = result & 0xF;
            }

            0x5 => {
                // AND R[N] &= R[operand]
                let val = self.registers[operand as usize] & self.registers[((operand + 1) % 8) as usize];
                self.flags_z = val == 0;
                self.registers[operand as usize] = val;
            }

            0x6 => {
                // OR R[N] |= R[operand]
                let val = self.registers[operand as usize] | self.registers[((operand + 1) % 8) as usize];
                self.flags_z = val == 0;
                self.registers[operand as usize] = val;
            }

            0x7 => {
                // XOR R[N] ^= R[operand]
                let val = self.registers[operand as usize] ^ self.registers[((operand + 1) % 8) as usize];
                self.flags_z = val == 0;
                self.registers[operand as usize] = val;
            }

            0x8 => {
                // JMP to address N
                self.pc = operand;
            }

            0x9 => {
                // JZ (jump if zero)
                if self.flags_z {
                    self.pc = operand;
                }
            }

            0xA => {
                // JNZ (jump if not zero)
                if !self.flags_z {
                    self.pc = operand;
                }
            }

            0xB => {
                // STORE RAM[R[operand]] <- R[N]
                let addr = self.registers[operand as usize] & 0xF;
                let val = self.registers[((operand + 1) % 8) as usize];
                self.ram[addr as usize] = val;
            }

            0xC => {
                // LOAD_R R[N] <- RAM[R[operand]]
                let addr = self.registers[operand as usize] & 0xF;
                self.registers[operand as usize] = self.ram[addr as usize];
            }

            0xD => {
                // OUT R[N] -> output
                self.output = self.registers[operand as usize] & 0xF;
            }

            0xE => {
                // IN (read external input into R[N] — placeholder, reads 0)
                self.registers[operand as usize] = 0;
            }

            0xF => {
                // HALT
                self.running = false;
            }

            _ => {}
        }

        self.cycle_count += 1;
    }

    /// Run until halt or max cycles
    pub fn run(&mut self, max_cycles: u64) {
        for _ in 0..max_cycles {
            if !self.running {
                break;
            }
            self.tick();
        }
    }

    /// Get output as redstone signal strength (0-15)
    pub fn signal_output(&self) -> u8 {
        self.output
    }
}

/// Example programs
pub fn example_counter() -> Vec<u8> {
    // Count from 0 to 15, output each value
    vec![
        0x10, 0x00, // LOAD R0, 0
        0x11, 0x01, // LOAD R1, 1
        0xD0,       // OUT R0
        0x30,       // ADD R0, R1  (R0 += R1)
        0x90,       // JZ to 0 (loop back when R0 wraps to 0)
        0x84,       // JMP to 4 (back to OUT)
        0xF0,       // HALT
    ]
}

pub fn example_fibonacci() -> Vec<u8> {
    // Fibonacci sequence mod 16
    vec![
        0x10, 0x00, // LOAD R0, 0  (a)
        0x11, 0x01, // LOAD R1, 1  (b)
        0x12, 0x00, // LOAD R2, 0  (temp)
        0xD0,       // OUT R0      (output a)
        0x22,       // MOV R2 <- R1 (temp = b)
        0x31,       // ADD R1, R0  (b = a + b)
        0x20,       // MOV R0 <- R2 (a = temp)
        0x83,       // JMP to 3    (loop)
        0xF0,       // HALT
    ]
}

use crate::memory::Memory;

const FLAG_Z: u8 = 0b10000000; // Zero
const FLAG_N: u8 = 0b01000000; // Subtraction
const FLAG_H: u8 = 0b00100000; // Half Carry
const FLAG_C: u8 = 0b00010000; // Carry

pub struct CPU {
    // 8bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,

    // 16bit registers
    pub sp: u16, // stack pointer
    pub pc: u16, // program counter
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xF0) as u8;
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    pub fn flag_z(&self) -> bool {
        (self.f & FLAG_Z) != 0
    }

    pub fn flag_c(&self) -> bool {
        (self.f & FLAG_C) != 0
    }

    pub fn flag_n(&self) -> bool {
        (self.f & FLAG_N) != 0
    }

    pub fn flag_h(&self) -> bool {
        (self.f & FLAG_H) != 0
    }

    pub fn set_flag_z(&mut self, value: bool) {
        if value {
            self.f |= FLAG_Z;
        } else {
            self.f &= !FLAG_Z;
        }
    }

    pub fn set_flag_c(&mut self, value: bool) {
        if value {
            self.f |= FLAG_C;
        } else {
            self.f &= !FLAG_C;
        }
    }

    pub fn set_flag_h(&mut self, value: bool) {
        if value {
            self.f |= FLAG_H;
        } else {
            self.f &= !FLAG_H;
        }
    }

    pub fn set_flag_n(&mut self, value: bool) {
        if value {
            self.f |= FLAG_N;
        } else {
            self.f &= !FLAG_N;
        }
    }

    pub fn execute_instruction(&mut self, opcode: u8, memory: &Memory) -> u8 {
        match opcode {
            0x00 => {
                // NOP
                4
            }
            0x3E => {
                // LD A, n - Load immediate value into A
                let value = memory.read_byte(self.pc);
                self.a = value;
                self.pc += 1;
                8
            }
            0x06 => {
                // LD B, n - Load immediate value into B
                let value = memory.read_byte(self.pc);
                self.b = value;
                self.pc += 1;
                8
            }
            0x3C => {
                // INC A - Increment A
                let old_a = self.a;
                let result = self.a.wrapping_add(1);
                self.a = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_a & 0x0F) == 0x0F);

                4
            }
            0x3D => {
                // DEC A - Decrement A
                let old_a = self.a;
                let result = old_a.wrapping_sub(1);
                self.a = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_a & 0x0F) == 0x00);

                4
            }
            0x04 => {
                // INC B - Increment B
                let old_b = self.b;
                let result = self.b.wrapping_add(1);
                self.b = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_b & 0x0F) == 0x0F);

                4
            }

            0x05 => {
                // DEC B - Decrement B
                let old_b = self.b;
                let result = self.b.wrapping_sub(1);
                self.b = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_b & 0x0F) == 0x00);

                4
            }
            _ => {
                println!("Opcode not implemented: 0x{:02X}", opcode);
                4
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        let cycles = cpu.execute_instruction(0x00, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_ld_a_immediate() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x100;
        memory.write_byte(0x100, 0x42);

        let cycles = cpu.execute_instruction(0x3E, &memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x101);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.b, 0x00);
    }

    #[test]
    fn test_ld_b_immediate() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x104;
        memory.write_byte(0x104, 0x0F);

        let cycles = cpu.execute_instruction(0x06, &memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x105);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.b, 0x0F);
    }

    #[test]
    fn test_inc_a() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3C, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_a_zero() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0xFF;

        let cycles = cpu.execute_instruction(0x3C, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_a_overflow() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0x0F;

        let cycles = cpu.execute_instruction(0x3C, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_a() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3D, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_zero() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0x01;

        let cycles = cpu.execute_instruction(0x3D, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_overflow() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.a = 0x10;

        let cycles = cpu.execute_instruction(0x3D, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x04, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_b_zero() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0xFF;

        let cycles = cpu.execute_instruction(0x04, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b_overflow() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0x0F;

        let cycles = cpu.execute_instruction(0x04, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_b() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x05, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_zero() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x05, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_overflow() {
        let mut cpu = CPU::new();
        let memory = Memory::new();

        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x05, &memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }
}

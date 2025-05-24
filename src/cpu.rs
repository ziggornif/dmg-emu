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

    // Interrupt Master Enable
    ime: bool,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
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
            ime: false,
        }
    }

    fn get_register(&self, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            7 => self.a,
            _ => panic!("Invalid index register {}", index),
        }
    }

    fn set_register(&mut self, index: u8, value: u8) {
        match index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            7 => self.a = value,
            _ => panic!("Invalid index register {}", index),
        }
    }

    fn alu_add(&mut self, value: u8) {
        let old_a = self.a;
        let result = old_a.wrapping_add(value);

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((old_a & 0x0F) + (value & 0x0F) > 0x0F);
        self.set_flag_c((old_a as u16) + (value as u16) > 0xFF);
    }

    fn alu_adc(&mut self, value: u8) {
        let old_a = self.a;
        let carry = if self.flag_c() { 1 } else { 0 };
        let result = old_a.wrapping_add(value).wrapping_add(carry);

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h((old_a & 0x0F) + (value & 0x0F) + carry > 0x0F);
        self.set_flag_c((old_a as u16) + (value as u16) + (carry as u16) > 0xFF);
    }

    fn alu_sub(&mut self, value: u8) {
        let old_a = self.a;
        let result = old_a.wrapping_sub(value);

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((old_a & 0x0F) < (value & 0x0F));
        self.set_flag_c((old_a as u16) < (value as u16));
    }

    fn alu_sbc(&mut self, value: u8) {
        let old_a = self.a;
        let carry = if self.flag_c() { 1 } else { 0 };
        let result = old_a.wrapping_sub(value).wrapping_sub(carry);

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((old_a & 0x0F) < (value & 0x0F) + carry);
        self.set_flag_c((old_a as u16) < (value as u16) + (carry as u16));
    }

    fn alu_and(&mut self, value: u8) {
        let result = self.a & value;

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(true);
        self.set_flag_c(false);
    }

    fn alu_xor(&mut self, value: u8) {
        let result = self.a ^ value;

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
    }

    fn alu_or(&mut self, value: u8) {
        let result = self.a | value;

        self.a = result;

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
    }

    fn alu_cp(&mut self, value: u8) {
        let old_a = self.a;
        let result = old_a.wrapping_sub(value);

        // Update flags
        self.set_flag_z(result == 0);
        self.set_flag_n(true);
        self.set_flag_h((old_a & 0x0F) < (value & 0x0F));
        self.set_flag_c((old_a as u16) < (value as u16));
    }

    fn stack_push(&mut self, memory: &mut Memory, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        memory.write_byte(self.sp, value as u8);
    }

    fn stack_pop(&mut self, memory: &mut Memory) -> u16 {
        let low = memory.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = memory.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (high << 8) | low
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

    pub fn interrupts_enabled(&self) -> bool {
        self.ime
    }

    pub fn execute_instruction(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        match opcode {
            0x00 => {
                // NOP
                4
            }
            0x01 => {
                // LD BC, nn - Load 16bits immediate into BC
                let low = memory.read_byte(self.pc);
                self.pc += 1;
                let high = memory.read_byte(self.pc);
                self.pc += 1;

                let value = ((high as u16) << 8) | (low as u16);
                self.set_bc(value);

                12
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
            0x3E => {
                // LD A, n - Load immediate value into A
                let value = memory.read_byte(self.pc);
                self.a = value;
                self.pc += 1;
                8
            }
            0x18 => {
                // JR r8 - Jump relative
                let offset = memory.read_byte(self.pc) as i8; // to get offset sign
                self.pc += 1;

                self.pc = ((self.pc as i32) + (offset as i32)) as u16;

                12
            }
            0x28 => {
                // JR Z, r8 - Jump relative if zero flag is set
                let offset = memory.read_byte(self.pc) as i8;
                self.pc += 1;

                if self.flag_z() {
                    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    12
                } else {
                    8
                }
            }
            0x40..=0x7F => {
                // LD r, r - Load register to register
                let dest_reg = (opcode >> 3) & 0x07;
                let src_reg = opcode & 0x07;

                // 0x76 = HALT
                if opcode == 0x76 {
                    println!("HALT instruction");
                    return 4;
                }

                // HL case - memory access
                if dest_reg == 6 || src_reg == 6 {
                    // TODO : implement LD (HL), r and LD r, (HL)
                    println!("LD (HL) not implemented: 0x{:02X}", opcode);
                    return 8;
                }

                // LD r, r
                let value = self.get_register(src_reg);
                self.set_register(dest_reg, value);

                4
            }
            0x80..=0xBF => {
                // ALU operations
                let operation = (opcode >> 3) & 0x07;
                let src_reg = opcode & 0x07;

                // HL case - memory access
                if src_reg == 6 {
                    // TODO : implement ALU (HL)
                    println!("ALU (HL) not implemented: 0x{:02X}", opcode);
                    return 8;
                }

                let src_value = self.get_register(src_reg);

                match operation {
                    0 => self.alu_add(src_value),
                    1 => self.alu_adc(src_value),
                    2 => self.alu_sub(src_value),
                    3 => self.alu_sbc(src_value),
                    4 => self.alu_and(src_value),
                    5 => self.alu_xor(src_value),
                    6 => self.alu_or(src_value),
                    7 => self.alu_cp(src_value),
                    _ => unreachable!(),
                }

                4
            }
            0xC5 => {
                // PUSH BC
                let value = self.bc();
                self.stack_push(memory, value);
                16
            }
            0xD5 => {
                // PUSH DE
                let value = self.de();
                self.stack_push(memory, value);
                16
            }
            0xE5 => {
                // PUSH HL
                let value = self.hl();
                self.stack_push(memory, value);
                16
            }
            0xF5 => {
                // PUSH AF
                let value = self.af();
                self.stack_push(memory, value);
                16
            }
            0xC1 => {
                // POP BC
                let value = self.stack_pop(memory);
                self.set_bc(value);
                12
            }
            0xD1 => {
                // POP DE
                let value = self.stack_pop(memory);
                self.set_de(value);
                12
            }
            0xE1 => {
                // POP HL
                let value = self.stack_pop(memory);
                self.set_hl(value);
                12
            }
            0xF1 => {
                // POP AF
                let value = self.stack_pop(memory);
                self.set_af(value);
                12
            }
            0xCD => {
                // CALL nn - Call function
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                // store current address
                self.stack_push(memory, self.pc);

                // jump to function address
                self.pc = address;

                24
            }
            0xC9 => {
                // RET - Return from function
                self.pc = self.stack_pop(memory);
                16
            }
            0xC3 => {
                // JP nn - Jump absolute
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                self.pc = address;

                16
            }
            0xC2 => {
                // JP NZ, nn
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                if !self.flag_z() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xCA => {
                // JP Z, nn
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                if self.flag_z() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xD2 => {
                // JP NC, nn
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                if !self.flag_c() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xDA => {
                // JP C, nn
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                if self.flag_c() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xC0 => {
                // RET NZ
                if !self.flag_z() {
                    self.pc = self.stack_pop(memory);
                    20
                } else {
                    8
                }
            }
            0xC8 => {
                // RET Z
                if self.flag_z() {
                    self.pc = self.stack_pop(memory);
                    20
                } else {
                    8
                }
            }
            0xD0 => {
                // RET NC
                if !self.flag_c() {
                    self.pc = self.stack_pop(memory);
                    20
                } else {
                    8
                }
            }
            0xD8 => {
                // RET C
                if self.flag_c() {
                    self.pc = self.stack_pop(memory);
                    20
                } else {
                    8
                }
            }
            0x20 => {
                // JR NZ, r8
                let offset = memory.read_byte(self.pc) as i8;
                self.pc += 1;

                if !self.flag_z() {
                    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    12
                } else {
                    8
                }
            }
            0x30 => {
                // JR NC, r8
                let offset = memory.read_byte(self.pc) as i8;
                self.pc += 1;

                if !self.flag_c() {
                    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    12
                } else {
                    8
                }
            }
            0x38 => {
                // JR C, r8
                let offset = memory.read_byte(self.pc) as i8;
                self.pc += 1;

                if self.flag_c() {
                    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    12
                } else {
                    8
                }
            }
            0xF3 => {
                // DI - Disable interrupts
                self.ime = false;

                4
            }
            0xFB => {
                // EI - Enable interrupts
                self.ime = true;

                4
            }
            0x31 => {
                // LD SP, nn - Load 16bits immediate into SP
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;

                self.sp = (high << 8) | low;

                12
            }
            0xD6 => {
                // SUB A, n - Substract immediate from A
                let value = memory.read_byte(self.pc);
                self.pc += 1;

                self.alu_sub(value);

                8
            }
            0xFE => {
                // CP A, n - Compare A with immediate
                let value = memory.read_byte(self.pc);
                self.pc += 1;

                self.alu_cp(value);

                8
            }
            0xE0 => {
                // LDH (n), A - Load A into 0xFF00+n
                let offset = memory.read_byte(self.pc);
                self.pc += 1;

                let address = 0xFF00 + (offset as u16);
                memory.write_byte(address, self.a);

                12
            }
            0xEA => {
                // LD (nn), A - Load A into absolute address
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;

                let address = (high << 8) | low;
                memory.write_byte(address, self.a);

                16
            }
            0x07 => {
                // RLCA - Rotate A left circular
                let carry = (self.a & 0x80) != 0;
                self.a = (self.a << 1) | if carry { 1 } else { 0 };

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(carry);

                4
            }
            0x0F => {
                // RRCA - Rotate A right circular
                let carry = (self.a & 0x01) != 0;
                self.a = (self.a >> 1) | if carry { 0x80 } else { 0 };

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(carry);

                4
            }
            0x03 => {
                // INC BC
                let value = self.bc().wrapping_add(1);
                self.set_bc(value);

                8
            }
            0x0B => {
                // DEC BC
                let value = self.bc().wrapping_sub(1);
                self.set_bc(value);

                8
            }
            0x21 => {
                // LD HL, nn - Load immediate into HL
                let low = memory.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = memory.read_byte(self.pc) as u16;
                self.pc += 1;

                let value = (high << 8) | low;
                self.set_hl(value);

                12
            }
            0x23 => {
                // INC HL
                let value = self.hl().wrapping_add(1);
                self.set_hl(value);

                8
            }
            0x2A => {
                // LD A, (HL+) - Load A from HL then increment HL
                let address = self.hl();
                self.a = memory.read_byte(address);

                let new_hl = address.wrapping_add(1);
                self.set_hl(new_hl);

                8
            }
            0xF0 => {
                // LDH A, n - Load A from 0xFF00+n
                let offset = memory.read_byte(self.pc);
                self.pc += 1;

                let address = 0xFF00 + (offset as u16);
                self.a = memory.read_byte(address);

                12
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
        let mut memory = Memory::new();

        let cycles = cpu.execute_instruction(0x00, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_ld_a_immediate() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x100;
        memory.write_byte(0x100, 0x42);

        let cycles = cpu.execute_instruction(0x3E, &mut memory);

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

        let cycles = cpu.execute_instruction(0x06, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x105);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.b, 0x0F);
    }

    #[test]
    fn test_inc_a() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3C, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_a_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xFF;

        let cycles = cpu.execute_instruction(0x3C, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_a_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x0F;

        let cycles = cpu.execute_instruction(0x3C, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_a() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3D, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x01;

        let cycles = cpu.execute_instruction(0x3D, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x10;

        let cycles = cpu.execute_instruction(0x3D, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x04, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_b_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0xFF;

        let cycles = cpu.execute_instruction(0x04, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0x0F;

        let cycles = cpu.execute_instruction(0x04, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_b() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x05, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x05, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x05, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_ld_bc_immediate() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x100;
        memory.write_byte(0x100, 0x42);
        memory.write_byte(0x101, 0x10);

        let cycles = cpu.execute_instruction(0x01, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x102);
        assert_eq!(cpu.bc(), 0x1042)
    }

    #[test]
    fn test_add_a_b() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x01;
        cpu.b = 0x02;

        let cycles = cpu.execute_instruction(0x80, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x03);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_add_a_b_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x00;
        cpu.b = 0x00;

        let cycles = cpu.execute_instruction(0x80, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_add_a_b_half_carry() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x0F;
        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x80, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_add_a_b_carry() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xF0;
        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x80, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_sub_a_b() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x0F;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x90, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0A);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sub_a_b_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x05;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x90, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sub_a_b_half_carry() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x10;
        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x90, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sub_a_b_carry() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x05;
        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x90, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xF5);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_jr_forward() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        memory.write_byte(0x0100, 0x05);

        let cycles = cpu.execute_instruction(0x18, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_backward() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        memory.write_byte(0x0100, 0xFC);

        let cycles = cpu.execute_instruction(0x18, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x00FD);
    }

    #[test]
    fn test_jr_z_taken() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);
        memory.write_byte(0x0100, 0x03);

        let cycles = cpu.execute_instruction(0x28, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0104);
    }

    #[test]
    fn test_jr_z_not_taken() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);
        memory.write_byte(0x0100, 0x03);

        let cycles = cpu.execute_instruction(0x28, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);
    }

    #[test]
    fn test_ld_b_a() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x42;
        cpu.b = 0x00;

        let cycles = cpu.execute_instruction(0x47, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x42);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_ld_a_c() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x00;
        cpu.c = 0x99;

        let cycles = cpu.execute_instruction(0x79, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x99);
        assert_eq!(cpu.c, 0x99);
    }

    #[test]
    fn test_ld_same_register() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.d = 0x33;

        let cycles = cpu.execute_instruction(0x52, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.d, 0x33);
    }

    #[test]
    fn test_ld_hl_memory() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.d = 0x33;

        let cycles = cpu.execute_instruction(0x70, &mut memory);

        assert_eq!(cycles, 8);
        // TODO: test LD (HL) case in the future
    }

    #[test]
    fn test_halt_instruction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        let cycles = cpu.execute_instruction(0x76, &mut memory);

        assert_eq!(cycles, 4);
        // TODO: test HALT state in the future
    }

    #[test]
    fn test_add_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x10;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x80, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x15);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_add_a_r_with_carry() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xFF;
        cpu.c = 0x01;

        let cycles = cpu.execute_instruction(0x81, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_adc_a_r_without_flag() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x10;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x88, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x15);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_adc_a_r_with_flag() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x20;
        cpu.d = 0x06;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x8A, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x27);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sub_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x20;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0x92, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sub_a_r_underflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x5;
        cpu.e = 0x16;

        let cycles = cpu.execute_instruction(0x93, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xEF);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_sbc_a_r_without_flag() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x10;
        cpu.e = 0x02;

        let cycles = cpu.execute_instruction(0x9B, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0E);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_sbc_a_r_with_flag() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x20;
        cpu.l = 0x05;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x9D, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x1A);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_and_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xF0;
        cpu.h = 0x0F;

        let cycles = cpu.execute_instruction(0xA4, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_xor_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xAA; // 1010 1010
        cpu.l = 0x55; // 0101 0101

        let cycles = cpu.execute_instruction(0xAD, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_xor_a_a_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xAA; // 1010 1010

        let cycles = cpu.execute_instruction(0xAF, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_or_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0xF0;
        cpu.b = 0x0F;

        let cycles = cpu.execute_instruction(0xB0, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_cp_a_r_equal() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x42;
        cpu.c = 0x42;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0xB9, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }
    #[test]
    fn test_cp_a_r() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.a = 0x42;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0xBA, &mut memory);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_push_pop_bc() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.set_bc(0x1234);
        cpu.sp = 0xFFFE;

        let cycles = cpu.execute_instruction(0xC5, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0xFFFC);
        assert_eq!(memory.read_byte(0xFFFC), 0x34);
        assert_eq!(memory.read_byte(0xFFFD), 0x12);

        // reset bc
        cpu.set_bc(0x0000);

        let cycles = cpu.execute_instruction(0xC1, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.bc(), 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_push_pop_de() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.set_de(0xABCD);
        cpu.sp = 0x8000;

        let cycles = cpu.execute_instruction(0xD5, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x7FFE);
        assert_eq!(memory.read_byte(0x7FFE), 0xCD);
        assert_eq!(memory.read_byte(0x7FFF), 0xAB);

        // reset de

        let cycles = cpu.execute_instruction(0xD1, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.de(), 0xABCD);
        assert_eq!(cpu.sp, 0x8000);
    }

    #[test]
    fn test_push_pop_hl() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.set_hl(0x5678);
        cpu.sp = 0x9000;

        let cycles = cpu.execute_instruction(0xE5, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x8FFE);
        assert_eq!(memory.read_byte(0x8FFE), 0x78);
        assert_eq!(memory.read_byte(0x8FFF), 0x56);

        cpu.set_hl(0x0000);

        let cycles = cpu.execute_instruction(0xE1, &mut memory);
        assert_eq!(cycles, 12);
        assert_eq!(cpu.hl(), 0x5678);
        assert_eq!(cpu.sp, 0x9000);
    }

    #[test]
    fn test_push_pop_af() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.set_af(0x1000);
        cpu.sp = 0x2000;

        let cycles = cpu.execute_instruction(0xF5, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x1FFE);
        assert_eq!(memory.read_byte(0x1FFE), 0x00);
        assert_eq!(memory.read_byte(0x1FFF), 0x10);

        cpu.set_af(0x0000);

        let cycles = cpu.execute_instruction(0xF1, &mut memory);
        assert_eq!(cycles, 12);
        assert_eq!(cpu.af(), 0x1000);
        assert_eq!(cpu.sp, 0x2000);
    }

    #[test]
    fn test_call_ret_basic() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.sp = 0xFFFE;

        // Setup CALL 0x0200
        memory.write_byte(0x0100, 0x00);
        memory.write_byte(0x0101, 0x02);

        let cycles = cpu.execute_instruction(0xCD, &mut memory);

        assert_eq!(cycles, 24);
        assert_eq!(cpu.pc, 0x0200);
        assert_eq!(cpu.sp, 0xFFFC);

        // assert return address is 0x0102
        assert_eq!(memory.read_byte(0xFFFC), 0x02);
        assert_eq!(memory.read_byte(0xFFFD), 0x01);

        let cycles = cpu.execute_instruction(0xC9, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_jp_absolute() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0150;

        memory.write_byte(0x0150, 0x00);
        memory.write_byte(0x0151, 0x03);

        let cycles = cpu.execute_instruction(0xC3, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_nz() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        memory.write_byte(0x0100, 0x00);
        memory.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xC2, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xC2, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_z() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        memory.write_byte(0x0100, 0x00);
        memory.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xCA, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xCA, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
    }

    #[test]
    fn test_jp_nc() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        memory.write_byte(0x0100, 0x00);
        memory.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xD2, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xD2, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_c() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        memory.write_byte(0x0100, 0x00);
        memory.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xDA, &mut memory);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xDA, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
    }

    #[test]
    fn test_ret_nz() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_z(true);

        memory.write_byte(0xFFFC, 0x34);
        memory.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xC0, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xC0, &mut memory);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_z() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_z(false);

        memory.write_byte(0xFFFC, 0x34);
        memory.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xC8, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_z(true);

        let cycles = cpu.execute_instruction(0xC8, &mut memory);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_nc() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_c(true);

        memory.write_byte(0xFFFC, 0x34);
        memory.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xD0, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xD0, &mut memory);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_c() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_c(false);

        memory.write_byte(0xFFFC, 0x34);
        memory.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xD8, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0xD8, &mut memory);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_jr_nz() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        memory.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x20, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0x20, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_nc() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        memory.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x30, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0x30, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_c() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        memory.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x38, &mut memory);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x38, &mut memory);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_not_implemented() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        let cycles = cpu.execute_instruction(0xFF, &mut memory);

        assert_eq!(cycles, 4);
    }
}

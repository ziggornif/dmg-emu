use crate::{bus::Bus, error, info};

const FLAG_Z: u8 = 0b10000000; // Zero
const FLAG_N: u8 = 0b01000000; // Subtraction
const FLAG_H: u8 = 0b00100000; // Half-Carry
const FLAG_C: u8 = 0b00010000; // Carry

const CB_RLC_CYCLES: u8 = 8;
const CB_RRC_CYCLES: u8 = 8;
const CB_RL_CYCLES: u8 = 8;
const CB_RR_CYCLES: u8 = 8;
const CB_SLA_CYCLES: u8 = 8;
const CB_SRA_CYCLES: u8 = 8;
const CB_SRL_CYCLES: u8 = 8;
const CB_SWAP_CYCLES: u8 = 8;

#[derive(Debug, Clone)]
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

    // HALT state
    pub halted: bool,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: 0xB0,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            ime: false,
            halted: false,
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

    pub fn stack_push(&mut self, bus: &mut Bus, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write_byte(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write_byte(self.sp, value as u8);
    }

    fn stack_pop(&mut self, bus: &mut Bus) -> u16 {
        let low = bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = bus.read_byte(self.sp) as u16;
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

    pub fn disable_interrupts(&mut self) {
        self.ime = false;
    }

    pub fn wake_from_halt(&mut self) {
        self.halted = false;
    }

    pub fn execute_instruction(&mut self, opcode: u8, bus: &mut Bus) -> u8 {
        if self.halted { return 4; }

        match opcode {
            0x00 => {
                // NOP
                4
            }
            0x01 => {
                // LD BC, nn - Load 16bits immediate into BC
                let value = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
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
                let value = bus.read_byte(self.pc);
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
                let value = bus.read_byte(self.pc);
                self.a = value;
                self.pc += 1;
                8
            }
            0x18 => {
                // JR r8 - Jump relative
                let offset = bus.read_byte(self.pc) as i8; // to get offset sign
                self.pc += 1;

                self.pc = ((self.pc as i32) + (offset as i32)) as u16;

                12
            }
            0x28 => {
                // JR Z, r8 - Jump relative if zero flag is set
                let offset = bus.read_byte(self.pc) as i8;
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
                    self.halted = true;
                    info!("CPU HALT executed at PC: 0x{:04X}", self.pc.wrapping_sub(1));
                    return 4;
                }

                if dest_reg == 6 {
                    let address = self.hl();
                    let value = self.get_register(src_reg);
                    bus.write_byte(address, value);
                    return 8;
                }

                if src_reg == 6 {
                    let address = self.hl();
                    let value = bus.read_byte(address);
                    self.set_register(dest_reg, value);
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

                let src_value = if src_reg == 6 {
                    // HL case - memory access
                    bus.read_byte(self.hl())
                } else {
                    self.get_register(src_reg)
                };

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

                if src_reg == 6 { 8 } else { 4 }
            }
            0xC5 => {
                // PUSH BC
                let value = self.bc();
                self.stack_push(bus, value);
                16
            }
            0xD5 => {
                // PUSH DE
                let value = self.de();
                self.stack_push(bus, value);
                16
            }
            0xE5 => {
                // PUSH HL
                let value = self.hl();
                self.stack_push(bus, value);
                16
            }
            0xF5 => {
                // PUSH AF
                let value = self.af();
                self.stack_push(bus, value);
                16
            }
            0xC1 => {
                // POP BC
                let value = self.stack_pop(bus);
                self.set_bc(value);
                12
            }
            0xD1 => {
                // POP DE
                let value = self.stack_pop(bus);
                self.set_de(value);
                12
            }
            0xE1 => {
                // POP HL
                let value = self.stack_pop(bus);
                self.set_hl(value);
                12
            }
            0xF1 => {
                // POP AF
                let value = self.stack_pop(bus);
                self.set_af(value);
                12
            }
            0xCD => {
                // CALL nn - Call function
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                // store current address
                self.stack_push(bus, self.pc);

                // jump to function address
                self.pc = address;

                24
            }
            0xC9 => {
                // RET - Return from function
                self.pc = self.stack_pop(bus);

                16
            }
            0xC3 => {
                // JP nn - Jump absolute
                let address = bus.read_word(self.pc);
                self.pc = address;

                16
            }
            0xC2 => {
                // JP NZ, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if !self.flag_z() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xCA => {
                // JP Z, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if self.flag_z() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xD2 => {
                // JP NC, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if !self.flag_c() {
                    self.pc = address;
                    16
                } else {
                    12
                }
            }
            0xDA => {
                // JP C, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

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
                    self.pc = self.stack_pop(bus);
                    20
                } else {
                    8
                }
            }
            0xC8 => {
                // RET Z
                if self.flag_z() {
                    self.pc = self.stack_pop(bus);
                    20
                } else {
                    8
                }
            }
            0xD0 => {
                // RET NC
                if !self.flag_c() {
                    self.pc = self.stack_pop(bus);
                    20
                } else {
                    8
                }
            }
            0xD8 => {
                // RET C
                if self.flag_c() {
                    self.pc = self.stack_pop(bus);
                    20
                } else {
                    8
                }
            }
            0x20 => {
                // JR NZ, r8
                let offset = bus.read_byte(self.pc) as i8;
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
                let offset = bus.read_byte(self.pc) as i8;
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
                let offset = bus.read_byte(self.pc) as i8;
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
                let value = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.sp = value;

                12
            }
            0xD6 => {
                // SUB A, n - Subtract immediate from A
                let value = bus.read_byte(self.pc);
                self.pc += 1;

                self.alu_sub(value);

                8
            }
            0xFE => {
                // CP A, n - Compare A with immediate
                let value = bus.read_byte(self.pc);
                self.pc += 1;

                self.alu_cp(value);

                8
            }
            0xE0 => {
                // LDH (n), A - Load A into 0xFF00+n
                let offset = bus.read_byte(self.pc);
                self.pc += 1;

                let address = 0xFF00 + (offset as u16);
                bus.write_byte(address, self.a);

                12
            }
            0xEA => {
                // LD (nn), A - Load A into absolute address
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
                bus.write_byte(address, self.a);

                16
            }
            0x07 => {
                // RLCA - Rotate A left circular
                let carry = (self.a & 0x80) != 0;
                self.a = (self.a << 1) | if carry { 1 } else { 0 };

                // Update flags
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

                // Update flags
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
                let value = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

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
                // LD A, (HL+) - Load A then increment HL
                let address = self.hl();
                self.a = bus.read_byte(address);

                let new_hl = address.wrapping_add(1);
                self.set_hl(new_hl);

                8
            }
            0xF0 => {
                // LDH A, n - Load A from 0xFF00+n
                let offset = bus.read_byte(self.pc);
                self.pc += 1;

                let address = 0xFF00 + (offset as u16);
                self.a = bus.read_byte(address);

                12
            }
            0xFA => {
                // LD (HL), A - Load A from absolute address
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.a = bus.read_byte(address);

                16
            }
            0x2C => {
                // INC L
                let old_l = self.l;
                let result = self.l.wrapping_add(1);
                self.l = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_l & 0x0F) == 0x0F);

                4
            }
            0xC4 => {
                // CALL NZ, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if !self.flag_z() {
                    self.stack_push(bus, self.pc);
                    self.pc = address;
                    24
                } else {
                    12
                }
            }
            0xCC => {
                // CALL Z, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if self.flag_z() {
                    self.stack_push(bus, self.pc);
                    self.pc = address;
                    24
                } else {
                    12
                }
            }
            0xD4 => {
                // CALL NC, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if !self.flag_c() {
                    self.stack_push(bus, self.pc);
                    self.pc = address;
                    24
                } else {
                    12
                }
            }
            0xDC => {
                // CALL C, nn
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);

                if self.flag_c() {
                    self.stack_push(bus, self.pc);
                    self.pc = address;
                    24
                } else {
                    12
                }
            }
            0x10 => {
                // STOP - Stop CPU until interrupt occurs
                let _next_byte = bus.read_byte(self.pc);
                self.pc += 1;

                4
            }
            0x0C => {
                // INC C
                let old_c = self.c;
                let result = self.c.wrapping_add(1);
                self.c = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_c & 0x0F) == 0x0F);

                4
            }
            0x14 => {
                // INC D
                let old_d = self.d;
                let result = self.d.wrapping_add(1);
                self.d = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_d & 0x0F) == 0x0F);

                4
            }
            0x1C => {
                // INC E
                let old_e = self.e;
                let result = self.e.wrapping_add(1);
                self.e = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_e & 0x0F) == 0x0F);

                4
            }
            0x24 => {
                // INC H
                let old_h = self.h;
                let result = self.h.wrapping_add(1);
                self.h = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((old_h & 0x0F) == 0x0F);

                4
            }
            0x0D => {
                // DEC C
                let old_c = self.c;
                let result = self.c.wrapping_sub(1);
                self.c = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_c & 0x0F) == 0x00);

                4
            }
            0x15 => {
                // DEC D
                let old_d = self.d;
                let result = self.d.wrapping_sub(1);
                self.d = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_d & 0x0F) == 0x00);

                4
            }
            0x1D => {
                // DEC E
                let old_e = self.e;
                let result = self.e.wrapping_sub(1);
                self.e = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_e & 0x0F) == 0x00);

                4
            }
            0x25 => {
                // DEC H
                let old_h = self.h;
                let result = self.h.wrapping_sub(1);
                self.h = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_h & 0x0F) == 0x00);

                4
            }
            0x2D => {
                // DEC L
                let old_l = self.l;
                let result = self.l.wrapping_sub(1);
                self.l = result;

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_l & 0x0F) == 0x00);

                4
            }
            0xE6 => {
                // AND A, n - Logical AND with immediate value
                let value = bus.read_byte(self.pc);
                self.pc += 1;

                self.alu_and(value);

                8
            }
            0x0E => {
                // LD C, n - Load immediate into C
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.c = value;

                8
            }
            0x11 => {
                // LD DE, nn - Load 16bit immediate into DE
                let value = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.set_de(value);

                12
            }
            0x08 => {
                // LD (nn), SP - Store stack pointer at absolute address
                let address = bus.read_word(self.pc);
                self.pc = self.pc.wrapping_add(2);
                bus.write_word(address, self.sp);

                20
            }
            0x1A => {
                // LD A, (DE) - Load A from memory at DE address
                let address = self.de();
                self.a = bus.read_byte(address);

                8
            }
            0x13 => {
                // INC DE - Increment 16bit DE register
                let value = self.de().wrapping_add(1);
                self.set_de(value);

                8
            }
            0x22 => {
                // LD (HL+), A - Store A at HL address then increment HL
                let address = self.hl();
                bus.write_byte(address, self.a);

                let new_hl = address.wrapping_add(1);
                self.set_hl(new_hl);

                8
            }
            0x33 => {
                // INC SP
                self.sp = self.sp.wrapping_add(1);

                8
            }
            0x1B => {
                // DEC DE
                let value = self.de().wrapping_sub(1);
                self.set_de(value);

                8
            }
            0x2B => {
                // DEC HL
                let value = self.hl().wrapping_sub(1);
                self.set_hl(value);

                8
            }
            0x3B => {
                // DEC SP
                self.sp = self.sp.wrapping_sub(1);

                8
            }
            0x0A => {
                // LD A, (BC) - Load A from memory at BC address
                let address = self.bc();
                self.a = bus.read_byte(address);

                8
            }
            0x02 => {
                // LD (BC), A - Store A at BC address
                let address = self.bc();
                bus.write_byte(address, self.a);

                8
            }
            0x12 => {
                // LD (DE), A - Store A at DE address
                let address = self.de();
                bus.write_byte(address, self.a);

                8
            }
            0x32 => {
                // LD (HL-), A - Store A at HL, then decrement HL
                let address = self.hl();
                bus.write_byte(address, self.a);

                let new_hl = address.wrapping_sub(1);
                self.set_hl(new_hl);

                8
            }
            0xC6 => {
                // ADD A, n - Add immediate value to A
                let value = bus.read_byte(self.pc);
                self.pc += 1;

                self.alu_add(value);

                8
            }
            0x26 => {
                // LD H, n - Load immediate value into H
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.h = value;

                8
            }
            0x29 => {
                // ADD HL, HL - Add HL to itself
                let hl_value = self.hl();
                let result = hl_value.wrapping_add(hl_value);
                self.set_hl(result);

                // Update flags
                self.set_flag_n(false);
                self.set_flag_h((hl_value & 0x0FFF) + (hl_value & 0x0FFF) > 0x0FFF);
                self.set_flag_c((hl_value as u32) + (hl_value as u32) > 0xFFFF);

                8
            }
            0x09 => {
                // ADD HL, BC
                let hl_value = self.hl();
                let bc_value = self.bc();
                let result = hl_value.wrapping_add(self.bc());
                self.set_hl(result);

                // Update flags
                self.set_flag_n(false);
                self.set_flag_h((hl_value & 0x0FFF) + (bc_value & 0x0FFF) > 0x0FFF);
                self.set_flag_c((hl_value as u32) + (bc_value as u32) > 0xFFFF);

                8
            }
            0x19 => {
                // ADD HL, DE
                let hl_value = self.hl();
                let de_value = self.de();
                let result = hl_value.wrapping_add(de_value);
                self.set_hl(result);

                // Update flags
                self.set_flag_n(false);
                self.set_flag_h((hl_value & 0x0FFF) + (de_value & 0x0FFF) > 0x0FFF);
                self.set_flag_c((hl_value as u32) + (de_value as u32) > 0xFFFF);

                8
            }
            0x39 => {
                // ADD HL, SP
                let hl_value = self.hl();
                let sp_value = self.sp;
                let result = hl_value.wrapping_add(sp_value);
                self.set_hl(result);

                // Update flags
                self.set_flag_n(false);
                self.set_flag_h((hl_value & 0x0FFF) + (sp_value & 0x0FFF) > 0x0FFF);
                self.set_flag_c((hl_value as u32) + (sp_value as u32) > 0xFFFF);

                8
            }
            0x16 => {
                // LD D, n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.d = value;

                8
            }
            0x1E => {
                // LD E, n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.e = value;

                8
            }
            0x2E => {
                // LD L, n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.l = value;

                8
            }
            0xCE => {
                // ADC A, n - Add with carry
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.alu_adc(value);

                8
            }
            0xDE => {
                // SBC A, n - Subtract with carry
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.alu_sbc(value);

                8
            }
            0xEE => {
                // XOR A, n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.alu_xor(value);

                8
            }
            0xF6 => {
                // OR A, n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                self.alu_or(value);

                8
            }
            0x1F => {
                // RRA - Rotate A right through carry
                let old_carry = if self.flag_c() { 1 } else { 0 };
                let new_carry = (self.a & 0x01) != 0;

                self.a = (self.a >> 1) | (old_carry << 7);

                // Update flags
                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(new_carry);

                4
            }
            0xCB => {
                let cb_opcode = bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.execute_cb(cb_opcode, bus)
            }
            0x35 => {
                // DEC (HL) - Decrement value at HL address
                let address = self.hl();
                let old_value = bus.read_byte(address);
                let result = old_value.wrapping_sub(1);

                bus.write_byte(address, result);

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h((old_value & 0x0F) == 0x00);

                12
            }
            0xE9 => {
                // JP (HL) - Jump to the address contained in HL
                self.pc = self.hl();
                4
            }

            0xFF => {
                // RST 38H - Call to 0x0038
                self.stack_push(bus, self.pc);
                self.pc = 0x0038;
                16
            }
            0x27 => {
                // DAA - Decimal Adjust Accumulator
                let mut a = self.a;

                if !self.flag_n() {
                    // Addition
                    if self.flag_h() || (a & 0x0F) > 0x09 {
                        a = a.wrapping_add(0x06);
                    }

                    if self.flag_c() || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        self.set_flag_c(true);
                    }
                } else {
                    // Subtraction
                    if self.flag_c() {
                        a = a.wrapping_sub(0x60);
                    }

                    if self.flag_h() {
                        a = a.wrapping_sub(0x06);
                    }
                }

                self.a = a;

                self.set_flag_z(a == 0);
                self.set_flag_h(false);

                4
            }
            0xF9 => {
                // LD SP, HL - Load HL into SP
                self.sp = self.hl();

                8
            }
            0x2F => {
                // CPL  - Complement A
                self.a = !self.a;

                // Update flags
                self.set_flag_n(true);
                self.set_flag_h(true);

                4
            }
            0x36 => {
                // LD (HL), n
                let value = bus.read_byte(self.pc);
                self.pc += 1;
                let addr = self.hl();
                bus.write_byte(addr, value);

                12
            }
            0xE8 => {
                // ADD SP, r8
                let offset = bus.read_byte(self.pc) as i8 as i16;
                self.pc = self.pc.wrapping_add(1);

                let sp_low = self.sp as u8;
                let offset_u8 = offset as u8;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h((sp_low & 0x0F) + (offset_u8 & 0x0F) > 0x0F);
                self.set_flag_c((sp_low as u16) + (offset_u8 as u16) > 0xFF);

                self.sp = (self.sp as i16).wrapping_add(offset) as u16;

                16
            }
            0xF8 => {
                // LD HL, SP+r8
                let offset = bus.read_byte(self.pc) as i8 as i16;
                self.pc = self.pc.wrapping_add(1);
                let result = (self.sp as i16).wrapping_add(offset) as u16;

                let sp_low = self.sp as u8;
                let offset_u8 = offset as u8;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h((sp_low & 0x0F) + (offset_u8 & 0x0F) > 0x0F);
                self.set_flag_c((sp_low as u16) + (offset_u8 as u16) > 0xFF);

                self.set_hl(result);

                12
            }
            0xE2 => {
                // LD (0xFF00+C), A - Store A at address 0xFF00 + C
                let address = 0xFF00 + self.c as u16;
                bus.write_byte(address, self.a);

                8
            }
            0xEF => {
                // RST 28H - Call address 0x0028
                self.stack_push(bus, self.pc);
                self.pc = 0x0028;

                16
            }
            0xDF => {
                // RST 18H - Call address 0x0018
                self.stack_push(bus, self.pc);
                self.pc = 0x0018;
                16
            }
            0x17 => {
                // RLA - Rotate A left through carry
                let carry = if self.flag_c() { 1 } else { 0 };
                let new_carry = (self.a & 0x80) != 0;
                self.a = (self.a << 1) | carry;

                // Update flags
                self.set_flag_z(false); // Z is always reset
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(new_carry);

                8
            }
            0x34 => {
                // INC (HL) - Increment value at HL address
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = value.wrapping_add(1);
                bus.write_byte(address, result);

                // Update flags
                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h((value & 0x0F) == 0x0F);

                12
            }
            0xD9 => {
                // RETI - Return from interrupt
                self.pc = self.stack_pop(bus);
                self.ime = true;

                16
            }
            0x37 => {
                // SCF - Set Carry Flag
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(true);

                4
            }
            0x3F => {
                // CCF - Complement Carry Flag
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(!self.flag_c());

                4
            }

            // RST instructions
            0xC7 => {
                // RST 00H - Call address 0x0000
                self.stack_push(bus, self.pc);
                self.pc = 0x0000;
                16
            }
            0xCF => {
                // RST 08H - Call address 0x0008
                self.stack_push(bus, self.pc);
                self.pc = 0x0008;
                16
            }
            0xD7 => {
                // RST 10H - Call address 0x0010
                self.stack_push(bus, self.pc);
                self.pc = 0x0010;
                16
            }
            0xE7 => {
                // RST 20H - Call address 0x0020
                self.stack_push(bus, self.pc);
                self.pc = 0x0020;
                16
            }
            0xF7 => {
                // RST 30H - Call address 0x0030
                self.stack_push(bus, self.pc);
                self.pc = 0x0030;
                16
            }

            // Additional LD instructions
            0xF2 => {
                // LD A, (0xFF00+C) - Load A from address 0xFF00 + C
                let address = 0xFF00 + self.c as u16;
                self.a = bus.read_byte(address);
                8
            }

            // Additional missing instructions that are commonly needed
            0x3A => {
                // LD A, (HL-) - Load A from HL, then decrement HL
                let address = self.hl();
                self.a = bus.read_byte(address);
                let new_hl = address.wrapping_sub(1);
                self.set_hl(new_hl);
                8
            }
            _ => {
                error!("Opcode not implemented: 0x{:02X}", opcode);

                4
            }
        }
    }

    // RLC r - Rotate Left Circular
    fn cb_rlc(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | if carry { 1 } else { 0 };
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(carry);
        result
    }

    // RRC r - Rotate Right Circular
    fn cb_rrc(&mut self, value: u8) -> u8 {
        let first = value & 0x01;
        let result = (value >> 1) | (first << 7);
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(first != 0);
        result
    }

    // RL r - Rotate Left through Carry
    fn cb_rl(&mut self, value: u8) -> u8 {
        let old_carry = if self.flag_c() { 1 } else { 0 };
        let new_carry = (value & 0x80) != 0;
        let result = (value << 1) | old_carry;
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(new_carry);
        result
    }

    // RR r - Rotate Right through Carry
    fn cb_rr(&mut self, value: u8) -> u8 {
        let old_carry = if self.flag_c() { 0x80 } else { 0 };
        let new_carry = (value & 0x01) != 0;
        let result = (value >> 1) | old_carry;
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(new_carry);
        result
    }

    // SLA r - Shift Left Arithmetic
    fn sla(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;
        let result = value << 1;
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(new_carry);
        result
    }

    // SRA r - Shift Right Arithmetic
    fn sra(&mut self, value: u8) -> u8 {
        let last = value & 0x80;
        let first = value & 0x01;
        let result = (value >> 1) | last;
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(first != 0);
        result
    }

    // SRL r - Shift Right Logical
    fn srl(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(new_carry);
        result
    }

    // SWAP r - Swap upper and lower 4 bits
    fn swap(&mut self, value: u8) -> u8 {
        let result = value.rotate_right(4);
        self.set_flag_z(value == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
        result
    }

    fn execute_cb(&mut self, cb_opcode: u8, bus: &mut Bus) -> u8 {
        match cb_opcode {
            // RLC r - Rotate Left Circular
            0x00 => {
                // RLC B
                self.b = self.cb_rlc(self.b);
                CB_RLC_CYCLES
            }
            0x01 => {
                // RLC C
                self.c = self.cb_rlc(self.c);
                CB_RLC_CYCLES
            }
            0x02 => {
                // RLC D
                self.d = self.cb_rlc(self.d);
                CB_RLC_CYCLES
            }
            0x03 => {
                // RLC E
                self.e = self.cb_rlc(self.e);
                CB_RLC_CYCLES
            }
            0x04 => {
                // RLC H
                self.h = self.cb_rlc(self.h);
                CB_RLC_CYCLES
            }
            0x05 => {
                // RLC L
                self.l = self.cb_rlc(self.l);
                CB_RLC_CYCLES
            }
            0x06 => {
                // RLC [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.cb_rlc(value);
                bus.write_byte(address, result);
                16
            }
            0x07 => {
                // RLC A
                self.a = self.cb_rlc(self.a);
                CB_RLC_CYCLES
            }

            // RRC r - Rotate Right Circular
            0x08 => {
                // RRC B
                self.b = self.cb_rrc(self.b);
                CB_RRC_CYCLES
            }
            0x09 => {
                // RRC C
                self.c = self.cb_rrc(self.c);
                CB_RRC_CYCLES
            }
            0x0A => {
                // RRC D
                self.d = self.cb_rrc(self.d);
                CB_RRC_CYCLES
            }
            0x0B => {
                // RRC E
                self.e = self.cb_rrc(self.e);
                CB_RRC_CYCLES
            }
            0x0C => {
                // RRC H
                self.h = self.cb_rrc(self.h);
                CB_RRC_CYCLES
            }
            0x0D => {
                // RRC L
                self.l = self.cb_rrc(self.l);
                CB_RRC_CYCLES
            }
            0x0E => {
                // RRC [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.cb_rrc(value);
                bus.write_byte(address, result);
                16
            }
            0x0F => {
                // RRC A
                self.a = self.cb_rrc(self.a);
                CB_RRC_CYCLES
            }

            // RL r - Rotate Left through Carry
            0x10 => {
                // RL B
                self.b = self.cb_rl(self.b);
                CB_RL_CYCLES
            }
            0x11 => {
                // RL C - Rotate C left through carry
                self.c = self.cb_rl(self.c);
                CB_RL_CYCLES
            }
            0x12 => {
                // RL D
                self.d = self.cb_rl(self.d);
                CB_RL_CYCLES
            }
            0x13 => {
                // RL E - Rotate E left through carry
                self.e = self.cb_rl(self.e);
                CB_RL_CYCLES
            }
            0x14 => {
                // RL H
                self.h = self.cb_rl(self.h);
                CB_RL_CYCLES
            }
            0x15 => {
                // RL L
                self.l = self.cb_rl(self.l);
                CB_RL_CYCLES
            }
            0x16 => {
                // RL [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.cb_rl(value);
                bus.write_byte(address, result);
                16
            }
            0x17 => {
                // RL A
                self.a = self.cb_rl(self.a);
                CB_RL_CYCLES
            }
            // RR r - Rotate Right through Carry
            0x18 => {
                // RR B
                self.b = self.cb_rr(self.b);
                CB_RR_CYCLES
            }

            0x19 => {
                // RR C
                self.c = self.cb_rr(self.c);
                CB_RR_CYCLES
            }

            0x1A => {
                // RR D
                self.d = self.cb_rr(self.d);
                CB_RR_CYCLES
            }

            0x1B => {
                // RR E
                self.e = self.cb_rr(self.e);
                CB_RR_CYCLES
            }
            0x1C => {
                // RR H
                self.h = self.cb_rr(self.h);
                CB_RR_CYCLES
            }
            0x1D => {
                // RR L
                self.l = self.cb_rr(self.l);
                CB_RR_CYCLES
            }
            0x1E => {
                // RR [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.cb_rr(value);
                bus.write_byte(address, result);
                16
            }
            0x1F => {
                // RR A
                self.a = self.cb_rr(self.a);
                CB_RR_CYCLES
            }

            // SLA r - Shift Left Arithmetic
            0x20 => {
                // SLA B
                self.b = self.sla(self.b);
                CB_SLA_CYCLES
            }
            0x21 => {
                // SLA C
                self.c = self.sla(self.c);
                CB_SLA_CYCLES
            }
            0x22 => {
                // SLA D
                self.d = self.sla(self.d);
                CB_SLA_CYCLES
            }
            0x23 => {
                // SLA E
                self.e = self.sla(self.e);
                CB_SLA_CYCLES
            }
            0x24 => {
                // SLA H
                self.h = self.sla(self.h);
                CB_SLA_CYCLES
            }
            0x25 => {
                // SLA L
                self.l = self.sla(self.l);
                CB_SLA_CYCLES
            }
            0x26 => {
                // SLA [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.sla(value);
                bus.write_byte(address, result);
                16
            }
            0x27 => {
                // SLA A
                self.a = self.sla(self.a);
                CB_SLA_CYCLES
            }

            // SRA r - Shift Right Arithmetic
            0x28 => {
                // SRA B
                self.b = self.sra(self.b);
                CB_SRA_CYCLES
            }
            0x29 => {
                // SRA C
                self.c = self.sra(self.c);
                CB_SRA_CYCLES
            }
            0x2A => {
                // SRA D
                self.d = self.sra(self.d);
                CB_SRA_CYCLES
            }
            0x2B => {
                // SRA E
                self.e = self.sra(self.e);
                CB_SRA_CYCLES
            }
            0x2C => {
                // SRA H
                self.h = self.sra(self.h);
                CB_SRA_CYCLES
            }
            0x2D => {
                // SRA L
                self.l = self.sra(self.l);
                CB_SRA_CYCLES
            }
            0x2E => {
                // SRA [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.sra(value);
                bus.write_byte(address, result);
                16
            }
            0x2F => {
                // SRA A
                self.a = self.sra(self.a);
                CB_SRA_CYCLES
            }

            // SWAP r - Swap upper and lower 4 bits
            0x30 => {
                // SWAP B
                self.b = self.swap(self.b);
                CB_SWAP_CYCLES
            }
            0x31 => {
                // SWAP C
                self.c = self.swap(self.c);
                CB_SWAP_CYCLES
            }
            0x32 => {
                // SWAP D
                self.d = self.swap(self.d);
                CB_SWAP_CYCLES
            }
            0x33 => {
                // SWAP E
                self.e = self.swap(self.e);
                CB_SWAP_CYCLES
            }
            0x34 => {
                // SWAP H
                self.h = self.swap(self.h);
                CB_SWAP_CYCLES
            }
            0x35 => {
                // SWAP L
                self.l = self.swap(self.l);
                CB_SWAP_CYCLES
            }
            0x36 => {
                // SWAP [HL]
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.swap(value);
                bus.write_byte(address, result);
                16
            }
            0x37 => {
                // SWAP A
                self.a = self.swap(self.a);
                CB_SWAP_CYCLES
            }

            // SRL r - Shift Right Logical
            0x38 => {
                // SRL B
                self.b = self.srl(self.b);
                CB_SRL_CYCLES
            }
            0x39 => {
                // SRL C
                self.c = self.srl(self.c);
                CB_SRL_CYCLES
            }
            0x3A => {
                // SRL D
                self.d = self.srl(self.b);
                CB_SRL_CYCLES
            }
            0x3B => {
                // SRL E
                self.e = self.srl(self.e);
                CB_SRL_CYCLES
            }
            0x3C => {
                // SRL H
                self.h = self.srl(self.h);
                CB_SRL_CYCLES
            }
            0x3D => {
                // SRL L
                self.l = self.srl(self.l);
                CB_SRL_CYCLES
            }
            0x3E => {
                // SRL HL
                let address = self.hl();
                let value = bus.read_byte(address);
                let result = self.srl(value);
                bus.write_byte(address, result);
                16
            }
            0x3F => {
                // SRL A
                self.a = self.srl(self.a);
                CB_SRL_CYCLES
            }

            0x40..=0x7F => {
                // BIT n, r - Test bit n in register r
                let bit_number = (cb_opcode >> 3) & 0x07;
                let register = cb_opcode & 0x07;

                let value = if register == 6 {
                    bus.read_byte(self.hl())
                } else {
                    self.get_register(register)
                };

                let bit_set = (value & (1 << bit_number)) != 0;

                // Update flags
                self.set_flag_z(!bit_set);
                self.set_flag_n(false);
                self.set_flag_h(true);

                if register == 6 { 12 } else { 8 }
            }

            0x80..=0xBF => {
                // RES n, r - Reset bit n in register r
                let bit_number = (cb_opcode >> 3) & 0x07;
                let register = cb_opcode & 0x07;

                if register == 6 {
                    let address = self.hl();
                    let value = bus.read_byte(address);
                    bus.write_byte(address, value & !(1 << bit_number));
                    16
                } else {
                    let value = self.get_register(register);
                    self.set_register(register, value & !(1 << bit_number));
                    8
                }
            }

            0xC0..=0xFF => {
                // SET n, r - Set bit n in register r
                let bit_number = (cb_opcode >> 3) & 0x07;
                let register = cb_opcode & 0x07;

                if register == 6 {
                    let address = self.hl();
                    let value = bus.read_byte(address);
                    bus.write_byte(address, value | (1 << bit_number));
                    16
                } else {
                    let value = self.get_register(register) | (1 << bit_number);
                    self.set_register(register, value);
                    8
                }
            }
        }
    }
}

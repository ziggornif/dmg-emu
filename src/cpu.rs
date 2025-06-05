use crate::bus::Bus;

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

    fn stack_push(&mut self, bus: &mut Bus, value: u16) {
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

        let result = (high << 8) | low;
        result
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

    pub fn execute_instruction(&mut self, opcode: u8, bus: &mut Bus) -> u8 {
        match opcode {
            0x00 => {
                // NOP
                4
            }
            0x01 => {
                // LD BC, nn - Load 16bits immediate into BC
                let low = bus.read_byte(self.pc);
                self.pc += 1;
                let high = bus.read_byte(self.pc);
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let address = (high << 8) | low;

                self.pc = address;

                16
            }
            0xC2 => {
                // JP NZ, nn
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                self.sp = (high << 8) | low;

                12
            }
            0xD6 => {
                // SUB A, n - Substract immediate from A
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                let address = (high << 8) | low;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                let address = (high << 8) | low;
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
                // CALL NZ, nn - Call function if not zero
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                let address = (high << 8) | low;

                if !self.flag_z() {
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
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                let value = (high << 8) | low;
                self.set_de(value);

                12
            }
            0x08 => {
                // LD (nn), SP - Store stack pointer at absolute address
                let low = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                let high = bus.read_byte(self.pc) as u16;
                self.pc += 1;

                let address = (high << 8) | low;

                bus.write_byte(address, self.sp as u8);
                bus.write_byte(address + 1, (self.sp >> 8) as u8);

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
                let hl_value = self.sp;
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
                // SBC A, n - Substract with carry
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
                self.pc += 1;

                match cb_opcode {
                    // RR r - Rotate Right through Carry
                    0x18 => {
                        // RR B
                        let old_carry = if self.flag_c() { 0x80 } else { 0 };
                        let new_carry = (self.b & 0x01) != 0;
                        self.b = (self.b >> 1) | old_carry;
                        self.set_flag_z(self.b == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(new_carry);
                        8
                    }

                    0x19 => {
                        // RR C
                        let old_carry = if self.flag_c() { 0x80 } else { 0 };
                        let new_carry = (self.c & 0x01) != 0;
                        self.c = (self.c >> 1) | old_carry;
                        self.set_flag_z(self.c == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(new_carry);
                        8
                    }

                    0x1A => {
                        // RR D
                        let old_carry = if self.flag_c() { 0x80 } else { 0 };
                        let new_carry = (self.d & 0x01) != 0;
                        self.d = (self.d >> 1) | old_carry;
                        self.set_flag_z(self.d == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(new_carry);
                        8
                    }

                    // SRL r - Shift Right Logical
                    0x38 => {
                        // SRL B
                        let new_carry = (self.b & 0x01) != 0;
                        self.b >>= 1;
                        self.set_flag_z(self.b == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(new_carry);
                        8
                    }

                    0x3F => {
                        // SRL A
                        let new_carry = (self.b & 0x01) != 0;
                        self.a >>= 1;
                        self.set_flag_z(self.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(new_carry);
                        8
                    }

                    0x37 => {
                        // SWAP A - Swap upper and lower 4 bits
                        self.a = (self.a << 4) | (self.a >> 4);
                        self.set_flag_z(self.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(false);
                        8
                    }

                    _ => {
                        println!("CB opcode not implemented: 0x{:02X}", cb_opcode);
                        8
                    }
                }
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
                // JP (HL) - Jump to address contained in HL
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
            _ => {
                println!("Opcode not implemented: 0x{:02X}", opcode);

                4
            }
        }
    }
}

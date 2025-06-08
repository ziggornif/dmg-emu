#[cfg(test)]
mod tests {
    use emulator::{bus::Bus, cpu::CPU};

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        let cycles = cpu.execute_instruction(0x00, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_ld_a_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x100;
        bus.write_byte(0x100, 0x42);

        let cycles = cpu.execute_instruction(0x3E, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x101);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.b, 0x00);
    }

    #[test]
    fn test_ld_b_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x104;
        bus.write_byte(0x104, 0x0F);

        let cycles = cpu.execute_instruction(0x06, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x105);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.b, 0x0F);
    }

    #[test]
    fn test_inc_a() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3C, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_a_zero() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0xFF;

        let cycles = cpu.execute_instruction(0x3C, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_a_overflow() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x0F;

        let cycles = cpu.execute_instruction(0x3C, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_a() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x0E;

        let cycles = cpu.execute_instruction(0x3D, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_zero() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x01;

        let cycles = cpu.execute_instruction(0x3D, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_a_overflow() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x10;

        let cycles = cpu.execute_instruction(0x3D, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x04, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_inc_b_zero() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0xFF;

        let cycles = cpu.execute_instruction(0x04, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_inc_b_overflow() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0x0F;

        let cycles = cpu.execute_instruction(0x04, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x10);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_dec_b() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0x0E;

        let cycles = cpu.execute_instruction(0x05, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0D);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_zero() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x05, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x00);
        assert_eq!(cpu.flag_z(), true);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), false);
    }

    #[test]
    fn test_dec_b_overflow() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x05, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
    }

    #[test]
    fn test_ld_bc_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x100;
        bus.write_byte(0x100, 0x42);
        bus.write_byte(0x101, 0x10);

        let cycles = cpu.execute_instruction(0x01, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x102);
        assert_eq!(cpu.bc(), 0x1042)
    }

    #[test]
    fn test_add_a_b() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x01;
        cpu.b = 0x02;

        let cycles = cpu.execute_instruction(0x80, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x00;
        cpu.b = 0x00;

        let cycles = cpu.execute_instruction(0x80, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x0F;
        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x80, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xF0;
        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x80, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x0F;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x90, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x05;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x90, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.b = 0x01;

        let cycles = cpu.execute_instruction(0x90, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x05;
        cpu.b = 0x10;

        let cycles = cpu.execute_instruction(0x90, &mut bus);

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
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        bus.write_byte(0x0100, 0x05);

        let cycles = cpu.execute_instruction(0x18, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_backward() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        bus.write_byte(0x0100, 0xFC);

        let cycles = cpu.execute_instruction(0x18, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x00FD);
    }

    #[test]
    fn test_jr_z_taken() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);
        bus.write_byte(0x0100, 0x03);

        let cycles = cpu.execute_instruction(0x28, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0104);
    }

    #[test]
    fn test_jr_z_not_taken() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);
        bus.write_byte(0x0100, 0x03);

        let cycles = cpu.execute_instruction(0x28, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);
    }

    #[test]
    fn test_ld_b_a() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x42;
        cpu.b = 0x00;

        let cycles = cpu.execute_instruction(0x47, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.b, 0x42);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_ld_a_c() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x00;
        cpu.c = 0x99;

        let cycles = cpu.execute_instruction(0x79, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x99);
        assert_eq!(cpu.c, 0x99);
    }

    #[test]
    fn test_ld_same_register() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.d = 0x33;

        let cycles = cpu.execute_instruction(0x52, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.d, 0x33);
    }

    #[test]
    fn test_ld_hl_memory() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.d = 0x33;

        let cycles = cpu.execute_instruction(0x70, &mut bus);

        assert_eq!(cycles, 8);
        // TODO: test LD (HL) case in the future
    }

    #[test]
    fn test_halt_instruction() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        let cycles = cpu.execute_instruction(0x76, &mut bus);

        assert_eq!(cycles, 4);
        // TODO: test HALT state in the future
    }

    #[test]
    fn test_add_a_r() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x80, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xFF;
        cpu.c = 0x01;

        let cycles = cpu.execute_instruction(0x81, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.b = 0x05;

        let cycles = cpu.execute_instruction(0x88, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x20;
        cpu.d = 0x06;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x8A, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x20;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0x92, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x5;
        cpu.e = 0x16;

        let cycles = cpu.execute_instruction(0x93, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.e = 0x02;

        let cycles = cpu.execute_instruction(0x9B, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x20;
        cpu.l = 0x05;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x9D, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xF0;
        cpu.h = 0x0F;

        let cycles = cpu.execute_instruction(0xA4, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xAA; // 1010 1010
        cpu.l = 0x55; // 0101 0101

        let cycles = cpu.execute_instruction(0xAD, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xAA; // 1010 1010

        let cycles = cpu.execute_instruction(0xAF, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0xF0;
        cpu.b = 0x0F;

        let cycles = cpu.execute_instruction(0xB0, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x42;
        cpu.c = 0x42;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0xB9, &mut bus);

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
        let mut bus = Bus::new();

        cpu.a = 0x42;
        cpu.d = 0x10;

        let cycles = cpu.execute_instruction(0xBA, &mut bus);

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
        let mut bus = Bus::new();

        cpu.set_bc(0x1234);
        cpu.sp = 0xFFFE;

        let cycles = cpu.execute_instruction(0xC5, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0xFFFC);
        assert_eq!(bus.read_byte(0xFFFC), 0x34);
        assert_eq!(bus.read_byte(0xFFFD), 0x12);

        // reset bc
        cpu.set_bc(0x0000);

        let cycles = cpu.execute_instruction(0xC1, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.bc(), 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_push_pop_de() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_de(0xABCD);
        cpu.sp = 0x8000;

        let cycles = cpu.execute_instruction(0xD5, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x7FFE);
        assert_eq!(bus.read_byte(0x7FFE), 0xCD);
        assert_eq!(bus.read_byte(0x7FFF), 0xAB);

        // reset de

        let cycles = cpu.execute_instruction(0xD1, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.de(), 0xABCD);
        assert_eq!(cpu.sp, 0x8000);
    }

    #[test]
    fn test_push_pop_hl() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_hl(0x5678);
        cpu.sp = 0x9000;

        let cycles = cpu.execute_instruction(0xE5, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x8FFE);
        assert_eq!(bus.read_byte(0x8FFE), 0x78);
        assert_eq!(bus.read_byte(0x8FFF), 0x56);

        cpu.set_hl(0x0000);

        let cycles = cpu.execute_instruction(0xE1, &mut bus);
        assert_eq!(cycles, 12);
        assert_eq!(cpu.hl(), 0x5678);
        assert_eq!(cpu.sp, 0x9000);
    }

    #[test]
    fn test_push_pop_af() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_af(0x1000);
        cpu.sp = 0x2000;

        let cycles = cpu.execute_instruction(0xF5, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.sp, 0x1FFE);
        assert_eq!(bus.read_byte(0x1FFE), 0x00);
        assert_eq!(bus.read_byte(0x1FFF), 0x10);

        cpu.set_af(0x0000);

        let cycles = cpu.execute_instruction(0xF1, &mut bus);
        assert_eq!(cycles, 12);
        assert_eq!(cpu.af(), 0x1000);
        assert_eq!(cpu.sp, 0x2000);
    }

    #[test]
    fn test_call_ret_basic() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.sp = 0xFFFE;

        // Setup CALL 0x0200
        bus.write_byte(0x0100, 0x00);
        bus.write_byte(0x0101, 0x02);

        let cycles = cpu.execute_instruction(0xCD, &mut bus);

        assert_eq!(cycles, 24);
        assert_eq!(cpu.pc, 0x0200);
        assert_eq!(cpu.sp, 0xFFFC);

        // assert return address is 0x0102
        assert_eq!(bus.read_byte(0xFFFC), 0x02);
        assert_eq!(bus.read_byte(0xFFFD), 0x01);

        let cycles = cpu.execute_instruction(0xC9, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_jp_absolute() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0150;

        bus.write_byte(0x0150, 0x00);
        bus.write_byte(0x0151, 0x03);

        let cycles = cpu.execute_instruction(0xC3, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_nz() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        bus.write_byte(0x0100, 0x00);
        bus.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xC2, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xC2, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_z() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        bus.write_byte(0x0100, 0x00);
        bus.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xCA, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);

        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xCA, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
    }

    #[test]
    fn test_jp_nc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        bus.write_byte(0x0100, 0x00);
        bus.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xD2, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xD2, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);
    }

    #[test]
    fn test_jp_c() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        bus.write_byte(0x0100, 0x00);
        bus.write_byte(0x0101, 0x03);

        let cycles = cpu.execute_instruction(0xDA, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0300);

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xDA, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
    }

    #[test]
    fn test_ret_nz() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_z(true);

        bus.write_byte(0xFFFC, 0x34);
        bus.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xC0, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0xC0, &mut bus);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_z() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_z(false);

        bus.write_byte(0xFFFC, 0x34);
        bus.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xC8, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_z(true);

        let cycles = cpu.execute_instruction(0xC8, &mut bus);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_nc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_c(true);

        bus.write_byte(0xFFFC, 0x34);
        bus.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xD0, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0xD0, &mut bus);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_c() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.sp = 0xFFFC;
        cpu.set_flag_c(false);

        bus.write_byte(0xFFFC, 0x34);
        bus.write_byte(0xFFFD, 0x12);

        let cycles = cpu.execute_instruction(0xD8, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFFFC);

        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0xD8, &mut bus);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_jr_nz() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_z(true);

        bus.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x20, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_z(false);

        let cycles = cpu.execute_instruction(0x20, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_nc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        bus.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x30, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        let cycles = cpu.execute_instruction(0x30, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_jr_c() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.set_flag_c(false);

        bus.write_byte(0x0100, 0x05); // offset

        let cycles = cpu.execute_instruction(0x38, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.pc, 0x0101);

        // reset pc
        cpu.pc = 0x0100;
        cpu.set_flag_c(true);

        let cycles = cpu.execute_instruction(0x38, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0106);
    }

    #[test]
    fn test_di() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        let cycles = cpu.execute_instruction(0xF3, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.interrupts_enabled(), false)
    }

    #[test]
    fn test_ei() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        let cycles = cpu.execute_instruction(0xFB, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.interrupts_enabled(), true)
    }

    #[test]
    fn test_sp_nn_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;

        bus.write_byte(0x0100, 0x34);
        bus.write_byte(0x0101, 0x12);

        let cycles = cpu.execute_instruction(0x31, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cpu.sp, 0x1234);
    }

    #[test]
    fn test_sub_a_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.pc = 0x0100;

        bus.write_byte(0x0100, 0x05);

        let cycles = cpu.execute_instruction(0xD6, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.a, 0x0B);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_cp_a_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.pc = 0x0100;

        bus.write_byte(0x0100, 0x05);

        let cycles = cpu.execute_instruction(0xFE, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.pc, 0x101);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), true);
        assert_eq!(cpu.flag_h(), true);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_ldh_a() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x10;
        cpu.pc = 0x0100;

        bus.write_byte(0x0100, 0x05);

        let cycles = cpu.execute_instruction(0xE0, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(bus.read_byte(0xFF05), cpu.a);
        assert_eq!(cpu.pc, 0x101);
    }

    #[test]
    fn test_ld_a_absolute() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        cpu.a = 0x15;

        bus.write_byte(0x0100, 0x34);
        bus.write_byte(0x0101, 0x12);

        let cycles = cpu.execute_instruction(0xEA, &mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(bus.read_byte(0x1234), cpu.a);
    }

    #[test]
    fn test_rlca() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0x0F;

        let cycles = cpu.execute_instruction(0x07, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x1E);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_rlca_carry() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0xF5;

        let cycles = cpu.execute_instruction(0x07, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xEB);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_rrca() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0xD6;

        let cycles = cpu.execute_instruction(0x0F, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0x6B);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_rrca_carry() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.a = 0xF5;

        let cycles = cpu.execute_instruction(0x0F, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.a, 0xFA);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_h(), false);
        assert_eq!(cpu.flag_c(), true);
    }

    #[test]
    fn test_inc_bc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_bc(0x1234);

        let cycles = cpu.execute_instruction(0x03, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.bc(), 0x1235);
    }

    #[test]
    fn test_dec_bc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_bc(0x1234);

        let cycles = cpu.execute_instruction(0x0B, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.bc(), 0x1233);
    }

    #[test]
    fn test_hl_nn_immediate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;

        bus.write_byte(0x0100, 0x34);
        bus.write_byte(0x0101, 0x12);

        let cycles = cpu.execute_instruction(0x21, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(cpu.hl(), 0x1234);
    }

    #[test]
    fn test_inc_hl() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_hl(0x1234);

        let cycles = cpu.execute_instruction(0x23, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.hl(), 0x1235);
    }

    #[test]
    fn test_lda_hl_inc() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.set_hl(0x0105);
        bus.write_byte(0x0105, 0x42);

        let cycles = cpu.execute_instruction(0x2A, &mut bus);

        assert_eq!(cycles, 8);
        assert_eq!(cpu.hl(), 0x0106);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_ldha() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        cpu.pc = 0x0100;
        bus.write_byte(0x0100, 0x05);
        bus.write_byte(0xFF05, 0x16);

        let cycles = cpu.execute_instruction(0xF0, &mut bus);

        assert_eq!(cycles, 12);
        assert_eq!(cpu.pc, 0x0101);
        assert_eq!(cpu.a, 0x16);
    }

    #[test]
    fn test_not_implemented() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new();

        let cycles = cpu.execute_instruction(0xFF, &mut bus);

        assert_eq!(cycles, 4);
    }
}

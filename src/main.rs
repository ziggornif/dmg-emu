use dmg_emu::{cpu::CPU, memory::Memory};

fn main() {
    println!("DMG EMU Booting ...");
    let mut cpu = CPU::new();
    let mut memory = Memory::new();

    load_program(&mut memory);

    cpu.pc = 0x0100; // cartridge boot address

    for _ in 0..20 {
        step(&mut cpu, &memory);
    }

    println!("End of execution");
    println!(
        "Final state - A: 0x{:02X}, B: 0x{:02X}, C: 0x{:02X}, D: 0x{:02X}, E: 0x{:02X}, H: 0x{:02X}, L: 0x{:02X}\nFlag Z: {}, Flag N: {}, Flag H: {}, Flag C: {}",
        cpu.a,
        cpu.b,
        cpu.c,
        cpu.d,
        cpu.e,
        cpu.h,
        cpu.l,
        cpu.flag_z(),
        cpu.flag_n(),
        cpu.flag_h(),
        cpu.flag_c()
    );
}

fn step(cpu: &mut CPU, memory: &Memory) {
    let opcode = memory.read_byte(cpu.pc);
    println!("PC: 0x{:04X}, OPCODE: 0x{:02X}", cpu.pc, opcode);

    cpu.pc += 1;

    let cycles = cpu.execute_instruction(opcode, memory);

    println!("   -> Cycles: {}", cycles);
}

// fn load_program(memory: &mut Memory) {
//     memory.write_byte(0x0100, 0x3E); // LD A, n
//     memory.write_byte(0x0101, 0x05); // A = 5
//     memory.write_byte(0x0102, 0x06); // LD B, n
//     memory.write_byte(0x0103, 0x01); // B = 1
//     memory.write_byte(0x0104, 0x90); // SUB A, B
//     memory.write_byte(0x0105, 0x28); // JR Z, offset
//     memory.write_byte(0x0106, 0x02); // offset +2
//     memory.write_byte(0x0107, 0x18); // JR offset
//     memory.write_byte(0x0108, 0xFB); // offset -5
//     memory.write_byte(0x0109, 0x00); // NOP
// }

// test LD opcodes
// fn load_program(memory: &mut Memory) {
//     memory.write_byte(0x0100, 0x3E); // LD A, n
//     memory.write_byte(0x0101, 0x42); // A = 0x42

//     memory.write_byte(0x0102, 0x47); // LD B, A  (0x47 = dest=0, src=7)
//     memory.write_byte(0x0103, 0x48); // LD C, B  (0x48 = dest=1, src=0)
//     memory.write_byte(0x0104, 0x51); // LD D, C  (0x51 = dest=2, src=1)
//     memory.write_byte(0x0105, 0x5A); // LD E, D  (0x5A = dest=3, src=2)
//     memory.write_byte(0x0106, 0x63); // LD H, E  (0x63 = dest=4, src=3)
//     memory.write_byte(0x0107, 0x6C); // LD L, H  (0x6C = dest=5, src=4)
//     memory.write_byte(0x0108, 0x7D); // LD A, L  (0x7D = dest=7, src=5)

//     memory.write_byte(0x0109, 0x00); // NOP
//     memory.write_byte(0x010A, 0x00); // NOP
// }

// test ALU
fn load_program(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E); // LD A, n
    memory.write_byte(0x0101, 0x0F); // A = 0x0F

    memory.write_byte(0x0102, 0x06); // LD B, n  
    memory.write_byte(0x0103, 0x01); // B = 0x01

    memory.write_byte(0x0104, 0x80); // ADD A, B → A = 0x0F + 0x01 = 0x10
    memory.write_byte(0x0105, 0x91); // SUB A, C → A = 0x10 - 0x00 = 0x10
    memory.write_byte(0x0106, 0xA0); // AND A, B → A = 0x10 & 0x01 = 0x00
    memory.write_byte(0x0107, 0x3E); // LD A, n
    memory.write_byte(0x0108, 0x0F); // A = 0x0F
    memory.write_byte(0x0109, 0xA8); // XOR A, B → A = 0x0F ^ 0x01 = 0x0E
    memory.write_byte(0x010A, 0xB0); // OR A, B  → A = 0x0E | 0x01 = 0x0F
    memory.write_byte(0x010B, 0xB8); // CP A, B  → Compare 0x0F avec 0x01 (A ne change pas)

    memory.write_byte(0x010C, 0x00); // NOP
}

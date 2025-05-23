use dmg_emu::{
    cpu::{self, CPU},
    memory::Memory,
};

fn main() {
    println!("DMG EMU Booting ...");
    let mut cpu = CPU::new();
    let mut memory = Memory::new();

    load_program(&mut memory);

    cpu.pc = 0x0100; // cartridge boot address

    for _ in 0..10 {
        step(&mut cpu, &memory);
    }

    println!("End of execution");
    println!("Final state - A: 0x{:02X}, B: 0x{:02X}", cpu.a, cpu.b);
}

fn step(cpu: &mut CPU, memory: &Memory) {
    let opcode = memory.read_byte(cpu.pc);
    println!("PC: 0x{:04X}, OPCODE: 0x{:02X}", cpu.pc, opcode);

    cpu.pc += 1;

    let cycles = cpu.execute_instruction(opcode, memory);

    println!("   -> Cycles: {}", cycles);
}

fn load_program(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E);
    memory.write_byte(0x0101, 0x42);
    memory.write_byte(0x0102, 0x3C);
    memory.write_byte(0x0103, 0x3C);
    memory.write_byte(0x0104, 0x00);
    memory.write_byte(0x0105, 0x00);
    memory.write_byte(0x106, 0x06);
    memory.write_byte(0x107, 0xFF);
    memory.write_byte(0x108, 0x04);
    memory.write_byte(0x109, 0x05);
}

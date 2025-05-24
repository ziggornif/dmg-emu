use std::fs;

use dmg_emu::{cpu::CPU, memory::Memory};

fn main() {
    println!("DMG EMU Booting ...");
    let mut cpu = CPU::new();
    let mut memory = Memory::new();

    match fs::read("resources/cpu_instrs.gb") {
        Ok(rom_data) => {
            println!("ROM loaded: {} bytes", rom_data.len());
            load_rom(&mut memory, &rom_data);
        }
        Err(_) => {
            panic!("ROM not found");
        }
    }

    cpu.pc = 0x0100; // cartridge boot address

    for _ in 0..1000 {
        step(&mut cpu, &mut memory);
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

fn step(cpu: &mut CPU, memory: &mut Memory) {
    let opcode = memory.read_byte(cpu.pc);
    println!("PC: 0x{:04X}, OPCODE: 0x{:02X}", cpu.pc, opcode);

    cpu.pc += 1;

    let cycles = cpu.execute_instruction(opcode, memory);

    println!("   -> Cycles: {}", cycles);
}

fn load_rom(memory: &mut Memory, rom_data: &[u8]) {
    for (i, &byte) in rom_data.iter().take(0x8000).enumerate() {
        memory.write_byte(i as u16, byte);
    }
}

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

    println!("Start ROM execution ...");
    for i in 0..50000 {
        let vblank = step(&mut cpu, &mut memory);

        if vblank {
            println!("V-Blank interrupt at step {}", i)
        }

        if i % 1000 == 0 {
            println!(
                "Step {}: PC=0x{:04X}, LY={}, PPU_MODE={:?}",
                i,
                cpu.pc,
                memory.ppu.ly,
                get_ppu_mode(&memory.ppu)
            );
        }
    }

    println!("End of execution");
    println!(
        "=== Final state ===\nVariables:\nA: 0x{:02X}, B: 0x{:02X}, C: 0x{:02X}, D: 0x{:02X}, E: 0x{:02X}, H: 0x{:02X}, L: 0x{:02X}\nFlags:\nZ: {}, Flag N: {}, Flag H: {}, Flag C: {}",
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

fn step(cpu: &mut CPU, memory: &mut Memory) -> bool {
    let opcode = memory.read_byte(cpu.pc);
    cpu.pc += 1;
    let cycles = cpu.execute_instruction(opcode, memory);
    memory.step_ppu(cycles)
}

fn load_rom(memory: &mut Memory, rom_data: &[u8]) {
    for (i, &byte) in rom_data.iter().take(0x8000).enumerate() {
        memory.write_byte(i as u16, byte);
    }
}

fn get_ppu_mode(ppu: &dmg_emu::ppu::PPU) -> &str {
    match ppu.stat & 0x03 {
        0 => "HBlank",
        1 => "VBlank",
        2 => "OAMScan",
        3 => "Drawing",
        _ => "Unknown",
    }
}

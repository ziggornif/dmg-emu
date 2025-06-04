use std::fs;

use dmg_emu::gameboy::Gameboy;

fn main() {
    println!("DMG EMU Booting ...");
    let mut gameboy = Gameboy::new();

    match fs::read("resources/cpu_instrs.gb") {
        Ok(rom_data) => match gameboy.load_rom(&rom_data) {
            Ok(_) => println!("ROM loaded successfully!"),
            Err(e) => {
                panic!("Error: {}", e);
            }
        },
        Err(_) => {
            panic!("ROM not found");
        }
    }

    // loop {
    //     gameboy.run_frame();
    //     // gameboy.print_debug_screen();

    //     std::thread::sleep(std::time::Duration::from_millis(16));
    // }
    loop {
        println!(
            "PPU Status: LCD={}, Mode={:?}, LY={}",
            gameboy.bus.ppu.is_lcd_enabled(),
            gameboy.bus.ppu.get_mode(),
            gameboy.bus.ppu.ly
        );
        // Logs pour les 100 premières instructions
        // if i < 100 {
        //     println!("=== Instruction {} ===", i);
        //     println!("PC: 0x{:04X}, SP: 0x{:04X}", gameboy.cpu.pc, gameboy.cpu.sp);

        //     let opcode = gameboy.bus.read_byte(gameboy.cpu.pc);
        //     println!("Opcode: 0x{:02X}", opcode);
        // }

        gameboy.step();
        gameboy.print_debug_screen();
        std::thread::sleep(std::time::Duration::from_millis(16));

        // if vblank {
        //     break;
        // }
    }
}

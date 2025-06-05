use std::fs;

use dmg_emu::gameboy::Gameboy;

fn main() {
    println!("DMG EMU Booting ...");
    let mut gameboy = Gameboy::new();

    match fs::read("resources/cpu_instrs.gb") {
        Ok(rom_data) => match gameboy.load_rom(&rom_data) {
            Ok(_) => println!("ROM loaded successfully!"),
            Err(e) => panic!("Error: {}", e),
        },
        Err(_) => {
            panic!("ROM not found");
        }
    }

    loop {
        gameboy.run_frame();
        gameboy.print_debug_screen();

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

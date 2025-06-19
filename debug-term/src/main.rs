use emulator::gameboy::Gameboy;
use std::fs;

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

    let mut frame_count = 0;

    loop {
        gameboy.run_frame();
        frame_count += 1;

        if frame_count % 60 == 0 {
            gameboy.print_debug_screen();
            println!("Frame: {}", frame_count);
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

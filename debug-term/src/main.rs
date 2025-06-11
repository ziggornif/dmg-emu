use emulator::gameboy::Gameboy;
use std::fs;

fn main() {
    println!("DMG EMU Booting ...");
    let mut gameboy = Gameboy::new();

    match fs::read("debug-term/resources/cpu_instrs.gb") {
        Ok(rom_data) => match gameboy.load_rom(&rom_data) {
            Ok(_) => println!("ROM loaded successfully!"),
            Err(e) => panic!("Error: {}", e),
        },
        Err(_) => {
            panic!("ROM not found");
        }
    }

    let mut frame_count = 0;
    let mut serial_buffer = String::new();


    loop {
        gameboy.run_frame();
        frame_count += 1;

        while let Some(serial_data) = gameboy.get_serial_output() {
            let ch = serial_data as char;
            if ch == '\n' || ch == '\r' {
                if !serial_buffer.is_empty() {
                    println!("Serial: {}", serial_buffer);
                    serial_buffer.clear();
                }
            } else if ch.is_ascii() && ch.is_control() == false {
                serial_buffer.push(ch);
            }
        }


        // Afficher seulement toutes les 60 frames (1 seconde à 60 FPS)
        if frame_count % 60 == 0 {
            gameboy.print_debug_screen();
            println!("Frame: {}", frame_count);
        }


        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
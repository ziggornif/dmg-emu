use emulator::gameboy::Gameboy;
use emulator::joypad::JoypadButton;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕹️ === TETRIS DEBUG MODE ===");
    println!("🎮 Commands:");
    println!("   w/s/a/d = Up/Down/Left/Right");
    println!("   space = A button");
    println!("   x = B button");
    println!("   enter = START");
    println!("   c = SELECT");
    println!("   r = Refresh screen");
    println!("   q = Quit");
    println!("   h = Help\n");

    let rom_data = std::fs::read("resources/tetris.gb")?;
    let mut gameboy = Gameboy::new();
    gameboy.load_rom(&rom_data)?;

    // Boot
    for _ in 0..60 { gameboy.run_frame(); }

    print_screen(&gameboy);

    loop {
        print!("\n🎮 Input: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "q" | "quit" => {
                println!("👋 Bye!");
                break;
            }
            "r" | "refresh" | "" => {
                print_screen(&gameboy);
            }
            "h" | "help" => {
                print_help();
            }
            _ => {
                if let Some(button) = parse_input(input) {
                    println!("🔹 Pressing {:?}...", button);

                    // Presser le bouton
                    gameboy.handle_input(button, true);
                    for _ in 0..20 { gameboy.run_frame(); }

                    // Relâcher
                    gameboy.handle_input(button, false);
                    for _ in 0..20 { gameboy.run_frame(); }

                    // Afficher le résultat
                    print_screen(&gameboy);
                } else {
                    println!("❌ Unknown command '{}'. Type 'h' for help.", input);
                }
            }
        }
    }

    Ok(())
}

fn parse_input(input: &str) -> Option<JoypadButton> {
    match input {
        "w" | "up" => Some(JoypadButton::Up),
        "s" | "down" => Some(JoypadButton::Down),
        "a" | "left" => Some(JoypadButton::Left),
        "d" | "right" => Some(JoypadButton::Right),
        " " | "space" => Some(JoypadButton::A),
        "x" => Some(JoypadButton::B),
        "enter" | "start" => Some(JoypadButton::Start),
        "c" | "select" => Some(JoypadButton::Select),
        _ => None,
    }
}

fn print_screen(gameboy: &Gameboy) {
    println!("\n🖥️ === TETRIS SCREEN ===");
    println!("PC: 0x{:04X}", gameboy.cpu.pc);
    gameboy.print_debug_screen();
    println!("=====================");
}

fn print_help() {
    println!("\n📖 HELP:");
    println!("  w = Up");
    println!("  s = Down");
    println!("  a = Left");
    println!("  d = Right");
    println!("  space = A button (rotate/select)");
    println!("  x = B button");
    println!("  enter = START button");
    println!("  c = SELECT button");
    println!("  r = Refresh screen");
    println!("  q = Quit");
}
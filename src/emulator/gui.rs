use crate::emulator::gameboy::Gameboy;
use crate::emulator::joypad::JoypadButton;
use eframe::egui;
use egui::{ColorImage, Key, TextureHandle, Vec2};

pub struct GameBoyApp {
    gameboy: Gameboy,
    texture: Option<TextureHandle>,
    scale: f32,
    show_debug: bool,
    paused: bool,
    frame_time: std::time::Instant,
    fps: f32,
    fps_counter: u32,
    fps_timer: std::time::Instant,
}

impl GameBoyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut gameboy = Gameboy::new();

        if let Ok(rom_data) = std::fs::read("resources/tetris.gb") {
            if let Err(e) = gameboy.load_rom(&rom_data) {
                eprintln!("Erreur lors du chargement de la ROM: {}", e);
            } else {
                // Boot rapide
                for _ in 0..60 {
                    gameboy.run_frame();
                }
            }
        } else {
            eprintln!("Impossible de charger resources/tetris.gb");
        }

        Self {
            gameboy,
            texture: None,
            scale: 3.0,
            show_debug: false,
            paused: false,
            frame_time: std::time::Instant::now(),
            fps: 0.0,
            fps_counter: 0,
            fps_timer: std::time::Instant::now(),
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let framebuffer = &self.gameboy.bus.ppu.framebuffer;
        let mut pixels = Vec::with_capacity(160 * 144 * 4);

        for row in framebuffer.iter() {
            for &pixel in row.iter() {
                let color = match pixel {
                    0 => [255, 255, 255, 255], // White
                    1 => [170, 170, 170, 255], // Light Gray
                    2 => [85, 85, 85, 255],    // Dark Gray
                    3 => [0, 0, 0, 255],       // Black
                    _ => [255, 0, 255, 255],   // Magenta debug
                };
                pixels.extend_from_slice(&color);
            }
        }

        let color_image = ColorImage::from_rgba_unmultiplied([160, 144], &pixels);

        if let Some(texture) = &mut self.texture {
            texture.set(color_image, egui::TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture(
                "gameboy_screen",
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Mapping des touches
        let keys = [
            (Key::W, JoypadButton::Up),
            (Key::S, JoypadButton::Down),
            (Key::A, JoypadButton::Left),
            (Key::D, JoypadButton::Right),
            (Key::Space, JoypadButton::A),
            (Key::X, JoypadButton::B),
            (Key::Enter, JoypadButton::Start),
            (Key::C, JoypadButton::Select),
        ];

        for (key, button) in keys.iter() {
            if input.key_pressed(*key) {
                self.gameboy.handle_input(*button, true);
            }
            if input.key_released(*key) {
                self.gameboy.handle_input(*button, false);
            }
        }

        // Touches de contrôle
        if input.key_pressed(Key::P) {
            self.paused = !self.paused;
        }
        if input.key_pressed(Key::F1) {
            self.show_debug = !self.show_debug;
        }
    }

    fn update_fps(&mut self) {
        self.fps_counter += 1;
        if self.fps_timer.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.fps_counter as f32;
            self.fps_counter = 0;
            self.fps_timer = std::time::Instant::now();
        }
    }
}

impl eframe::App for GameBoyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);

        if !self.paused && self.frame_time.elapsed().as_millis() >= 8 {
            for _ in 0..2 {
                self.gameboy.run_frame();
            }
            self.frame_time = std::time::Instant::now();
            self.update_fps();
        }

        self.update_texture(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🕹️ Game Boy Emulator");
                ui.separator();

                if ui
                    .button(if self.paused {
                        "▶ Resume"
                    } else {
                        "⏸ Pause"
                    })
                    .clicked()
                {
                    self.paused = !self.paused;
                }

                ui.separator();
                ui.label(format!("FPS: {:.1}", self.fps));

                ui.separator();
                ui.checkbox(&mut self.show_debug, "Debug");
            });

            ui.separator();

            if let Some(texture) = &self.texture {
                let screen_size = Vec2::new(160.0 * self.scale, 144.0 * self.scale);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.image((texture.id(), screen_size));

                        // Contrôles de zoom
                        ui.horizontal(|ui| {
                            ui.label("Zoom:");
                            if ui.button("1x").clicked() {
                                self.scale = 1.0;
                            }
                            if ui.button("2x").clicked() {
                                self.scale = 2.0;
                            }
                            if ui.button("3x").clicked() {
                                self.scale = 3.0;
                            }
                            if ui.button("4x").clicked() {
                                self.scale = 4.0;
                            }
                        });
                    });

                    if self.show_debug {
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.heading("Debug Info");

                            ui.label(format!("PC: 0x{:04X}", self.gameboy.cpu.pc));
                            ui.label(format!("SP: 0x{:04X}", self.gameboy.cpu.sp));
                            ui.separator();

                            ui.label("Registers:");
                            ui.label(format!(
                                "A: 0x{:02X}  F: 0x{:02X}",
                                self.gameboy.cpu.a, self.gameboy.cpu.f
                            ));
                            ui.label(format!("BC: 0x{:04X}", self.gameboy.cpu.bc()));
                            ui.label(format!("DE: 0x{:04X}", self.gameboy.cpu.de()));
                            ui.label(format!("HL: 0x{:04X}", self.gameboy.cpu.hl()));

                            ui.separator();
                            ui.label("Flags:");
                            ui.label(format!(
                                "Z:{} N:{} H:{} C:{}",
                                if self.gameboy.cpu.flag_z() { "1" } else { "0" },
                                if self.gameboy.cpu.flag_n() { "1" } else { "0" },
                                if self.gameboy.cpu.flag_h() { "1" } else { "0" },
                                if self.gameboy.cpu.flag_c() { "1" } else { "0" }
                            ));

                            ui.separator();
                            ui.label("PPU:");
                            ui.label(format!("Mode: {:?}", self.gameboy.bus.ppu.get_mode()));
                            ui.label(format!("LY: {}", self.gameboy.bus.ppu.ly));
                            ui.label(format!("LCDC: 0x{:02X}", self.gameboy.bus.ppu.lcdc));
                            ui.label(format!("STAT: 0x{:02X}", self.gameboy.bus.ppu.stat));
                            ui.label(format!(
                                "LCD: {}",
                                if self.gameboy.bus.ppu.is_lcd_enabled() {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            ));

                            ui.separator();
                            if ui.button("Print Terminal Screen").clicked() {
                                println!("\n=== Debug Screen Print ===");
                                self.gameboy.print_debug_screen();
                                println!("=========================");
                            }
                        });
                    }
                });
            }

            // Instructions
            ui.separator();
            ui.collapsing("🎮 Controls", |ui| {
                ui.label("• WASD: D-Pad");
                ui.label("• Space: A Button");
                ui.label("• X: B Button");
                ui.label("• Enter: Start");
                ui.label("• C: Select");
                ui.label("• P: Pause/Resume");
                ui.label("• F1: Toggle Debug");
            });

            ui.collapsing("🔧 Debug Actions", |ui| {
                if ui.button("Print Full Screen to Terminal").clicked() {
                    println!("\n=== FULL SCREEN DEBUG ===");
                    self.gameboy.print_debug_screen();
                    println!("========================");
                }

                if ui.button("Print CPU State to Terminal").clicked() {
                    println!("\n=== CPU STATE DEBUG ===");
                    crate::print_cpu_state!(self.gameboy.cpu);
                    println!("======================");
                }

                if ui.button("Print PPU State to Terminal").clicked() {
                    println!("\n=== PPU STATE DEBUG ===");
                    crate::print_ppu_state!(self.gameboy.bus.ppu);
                    println!("======================");
                }

                if ui.button("Debug OAM Sprites").clicked() {
                    println!("\n=== OAM SPRITES DEBUG ===");
                    for i in 0..40 {
                        let base = 0xFE00 + (i * 4);
                        let y = self.gameboy.bus.memory.read_oam(base);
                        let x = self.gameboy.bus.memory.read_oam(base + 1);
                        let tile = self.gameboy.bus.memory.read_oam(base + 2);
                        let flags = self.gameboy.bus.memory.read_oam(base + 3);

                        if y != 0 && x != 0 {
                            println!(
                                "Sprite {}: Y={} X={} Tile=0x{:02X} Flags=0x{:02X}",
                                i, y, x, tile, flags
                            );
                        }
                    }
                    println!("========================");
                }

                if ui.button("Force 60 Frames").clicked() {
                    println!("🔄 Forcing 60 frames...");
                    for i in 0..60 {
                        self.gameboy.run_frame();
                        if i % 10 == 0 {
                            println!("Frame {}/60", i);
                        }
                    }
                    println!("✅ Done!");
                }
            });
        });

        ctx.request_repaint();
    }
}

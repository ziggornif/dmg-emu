use crate::emulator::gameboy::Gameboy;
use crate::emulator::joypad::JoypadButton;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use egui::{ColorImage, Key, TextureHandle, Vec2};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct GameBoyApp {
    gameboy: Gameboy,
    texture: Option<TextureHandle>,
    scale: f32,
    show_debug: bool,
    paused: bool,
    last_update: Instant,
    frame_accumulator: Duration,
    fps: f32,
    fps_counter: u32,
    fps_timer: Instant,
    audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,
    _audio_stream: Option<cpal::Stream>,
    muted: bool,
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

        let audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>> = Arc::new(Mutex::new(VecDeque::new()));
        let audio_stream = Self::init_audio_stream(Arc::clone(&audio_buffer));

        Self {
            gameboy,
            texture: None,
            scale: 3.0,
            show_debug: false,
            paused: false,
            last_update: Instant::now(),
            frame_accumulator: Duration::ZERO,
            fps: 0.0,
            fps_counter: 0,
            fps_timer: Instant::now(),
            audio_buffer,
            _audio_stream: audio_stream,
            muted: false,
        }
    }

    fn init_audio_stream(audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>>) -> Option<cpal::Stream> {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("No audio output device found");
                return None;
            }
        };

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: 44100,
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = audio_buffer;
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock().unwrap();
                    for frame in data.chunks_mut(2) {
                        if let Some((left, right)) = buf.pop_front() {
                            frame[0] = left;
                            frame[1] = right;
                        } else {
                            frame[0] = 0.0;
                            frame[1] = 0.0;
                        }
                    }
                },
                |err| {
                    eprintln!("Audio stream error: {}", err);
                },
                None,
            )
            .ok();

        if let Some(ref s) = stream
            && let Err(e) = s.play()
        {
            eprintln!("Failed to start audio stream: {}", e);
        }

        stream
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
        if input.key_pressed(Key::M) {
            self.muted = !self.muted;
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

        if !self.paused {
            // Game Boy frame duration: ~16.74ms (59.7275 Hz)
            const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);
            // Cap to avoid spiral of death if emulation falls behind
            const MAX_CATCHUP_FRAMES: u32 = 4;

            let now = Instant::now();
            self.frame_accumulator += now - self.last_update;
            self.last_update = now;

            let mut frames_run = 0u32;
            while self.frame_accumulator >= FRAME_DURATION && frames_run < MAX_CATCHUP_FRAMES {
                self.gameboy.run_frame();
                self.frame_accumulator -= FRAME_DURATION;
                frames_run += 1;
                self.update_fps();
            }

            // If still behind after max catchup, reset to avoid permanent lag
            if self.frame_accumulator >= FRAME_DURATION {
                self.frame_accumulator = Duration::ZERO;
            }

            // Drain APU samples into shared audio buffer
            if frames_run > 0 {
                let samples = self.gameboy.take_audio_samples();
                if !self.muted
                    && !samples.is_empty()
                    && let Ok(mut buf) = self.audio_buffer.lock()
                {
                    const MAX_BUFFER: usize = 44100;
                    let available = MAX_BUFFER.saturating_sub(buf.len());
                    buf.extend(samples.into_iter().take(available));
                }
            }
        }

        self.update_texture(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("DMG-EMU - Game Boy Emulator");
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
                if ui
                    .button(if self.muted { "Unmute" } else { "Mute" })
                    .clicked()
                {
                    self.muted = !self.muted;
                }

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
                            ui.label("APU:");
                            let nr52 = self.gameboy.bus.apu.read_register(0xFF26);
                            ui.label(format!(
                                "Power: {}",
                                if nr52 & 0x80 != 0 { "ON" } else { "OFF" }
                            ));
                            ui.label(format!(
                                "NR50: 0x{:02X}  NR51: 0x{:02X}",
                                self.gameboy.bus.apu.nr50, self.gameboy.bus.apu.nr51
                            ));
                            ui.label(format!(
                                "CH1:{} CH2:{} CH3:{} CH4:{}",
                                if self.gameboy.bus.apu.channel1.enabled {
                                    "ON"
                                } else {
                                    "OFF"
                                },
                                if self.gameboy.bus.apu.channel2.enabled {
                                    "ON"
                                } else {
                                    "OFF"
                                },
                                if self.gameboy.bus.apu.channel3.enabled {
                                    "ON"
                                } else {
                                    "OFF"
                                },
                                if self.gameboy.bus.apu.channel4.enabled {
                                    "ON"
                                } else {
                                    "OFF"
                                },
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
                ui.label("• M: Mute/Unmute");
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

                if ui.button("Print APU State to Terminal").clicked() {
                    println!("\n=== APU STATE DEBUG ===");
                    crate::print_apu_state!(self.gameboy.bus.apu);
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

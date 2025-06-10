use crate::memory::Memory;

#[derive(Debug, Clone)]
pub struct PPU {
    // PPU registers
    pub lcdc: u8, // LCD Control
    pub stat: u8, // LCD Status
    pub scy: u8,  // Scroll Y
    pub scx: u8,  // Scroll X
    pub ly: u8,   // LCD Y Coord
    pub lyc: u8,  // LY Compare
    pub wy: u8,   // Window Y Pos
    pub wx: u8,   // Window X Pos

    // palette
    pub bgp: u8,  // Background palette
    pub obp0: u8, // Object palette 0
    pub obp1: u8, // Object palette 1

    cycles: u32,
    mode: PPUMode,

    // Framebuffer
    pub framebuffer: [[u8; 160]; 144], // 160x144px
}

impl Default for PPU {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PPUMode {
    HBLank = 0,
    VBlank = 1,
    OAMScan = 2,
    Drawing = 3,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            lcdc: 0x91,
            stat: 0x00,
            scy: 0x00,
            scx: 0x00,
            ly: 0,
            lyc: 0,
            wy: 0x00,
            wx: 0x00,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            cycles: 0,
            mode: PPUMode::OAMScan,
            framebuffer: [[0; 160]; 144],
        }
    }

    pub fn get_cycles(&self) -> u32 {
        self.cycles
    }

    pub fn get_mode(&self) -> PPUMode {
        self.mode
    }

    pub fn can_access_vram(&self) -> bool {
        (self.lcdc & 0x80) == 0 || self.mode != PPUMode::Drawing
    }

    pub fn can_access_oam(&self) -> bool {
        (self.lcdc & 0x80) == 0 || !matches!(self.mode, PPUMode::OAMScan | PPUMode::Drawing)
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | (self.mode as u8),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            _ => {
                println!("Register not implemented: 0x{:04X}", address);
                0xFF
            }
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => {
                let lcd_enabled_before = self.is_lcd_enabled();
                self.lcdc = value;
                let lcd_enabled_after = self.is_lcd_enabled();

                if lcd_enabled_before && !lcd_enabled_after {
                    self.ly = 0;
                    self.cycles = 0;
                    self.mode = PPUMode::HBLank;
                }
            }
            0xFF41 => self.stat = (self.stat & 0x87) | (value & 0x78), // (current & 1000 0111) | (new_val & 0111 1000)
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // readonly
            0xFF45 => self.lyc = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            0xFF47 => {
                self.bgp = value;
                self.decode_palette(value, "Background");
            }
            0xFF48 => {
                self.obp0 = value;
                self.decode_palette(value, "Object 0");
            }
            0xFF49 => {
                self.obp1 = value;
                self.decode_palette(value, "Object 1");
            }
            _ => {
                println!(
                    "Register not implemented: 0x{:04X} = 0x{:02X}",
                    address, value
                );
            }
        }
    }

    pub fn step(&mut self, cpu_cycles: u8, memory: &Memory) -> bool {
        if (self.lcdc & 0x80) == 0 {
            return false;
        }

        self.cycles += cpu_cycles as u32;
        let mut vblank_interrupt = false;

        match self.mode {
            PPUMode::OAMScan => {
                if self.cycles >= 80 {
                    // println!(
                    //     "OAMScan complete! {} -> Drawing (LY={})",
                    //     self.cycles, self.ly
                    // );
                    self.cycles -= 80;
                    self.mode = PPUMode::Drawing;
                } else {
                    // println!("OAMScan: {} / 80 cycles", self.cycles);
                }
            }
            PPUMode::Drawing => {
                if self.cycles >= 172 {
                    // println!(
                    //     "Drawing complete! {} -> HBlank (LY={})",
                    //     self.cycles, self.ly
                    // );
                    self.cycles -= 172;

                    self.render_line(memory);

                    self.mode = PPUMode::HBLank;
                } else {
                    // println!("Drawing: {} / 172 cycles", self.cycles);
                }
            }
            PPUMode::HBLank => {
                if self.cycles >= 204 {
                    // println!("HBlank complete! LY {} -> {}", self.ly, self.ly + 1);
                    self.cycles -= 204;
                    self.ly += 1;

                    if self.ly >= 144 {
                        self.mode = PPUMode::VBlank;
                        vblank_interrupt = true;
                    } else {
                        self.mode = PPUMode::OAMScan;
                    }
                } else {
                    // println!(
                    //     "HBlank: {} / 204 cycles (need {} more)",
                    //     self.cycles,
                    //     204 - self.cycles
                    // );
                }
            }
            PPUMode::VBlank => {
                // println!("VBlank processing: LY={}, cycles={}", self.ly, self.cycles);
                if self.cycles >= 456 {
                    // println!("VBlank line complete! LY {} → {}", self.ly, self.ly + 1);
                    self.cycles -= 456;
                    self.ly += 1;

                    if self.ly >= 154 {
                        // println!("VBlank finished! Resetting to LY=0, OAMScan");
                        self.ly = 0;
                        self.mode = PPUMode::OAMScan;
                    }
                }
            }
        }

        if self.ly == self.lyc {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }

        vblank_interrupt
    }

    pub fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & 0x80) != 0
    }

    // fn render_line(&mut self) {
    //     let line = self.ly as usize;
    //     if line >= 144 {
    //         return;
    //     }

    //     for x in 0..160 {
    //         let color = match (x / 40, line / 36) {
    //             (0, 0) => 0, // White
    //             (1, 0) => 1, // Light Gray
    //             (2, 0) => 2, // Dark Gray
    //             (3, 0) => 3, // Black
    //             (0, 1) => 1,
    //             (1, 1) => 2,
    //             (2, 1) => 3,
    //             (3, 1) => 0,
    //             (0, 2) => 2,
    //             (1, 2) => 3,
    //             (2, 2) => 0,
    //             (3, 2) => 1,
    //             (0, 3) => 3,
    //             (1, 3) => 0,
    //             (2, 3) => 1,
    //             (3, 3) => 2,
    //             _ => (x + line) % 4, // Default pattern
    //         };

    //         self.framebuffer[line][x] = color as u8;
    //     }
    // }

    // Exemple de vraie fonction render_line() qui lit la VRAM
    fn render_line(&mut self, memory: &Memory) {
        let line = self.ly as usize;
        if line >= 144 {
            return;
        }

        // Vérifier si le background est activé
        if (self.lcdc & 0x01) == 0 {
            // Background désactivé - remplir de blanc
            for x in 0..160 {
                self.framebuffer[line][x] = 0;
            }
            return;
        }

        // Calculer la ligne du background avec scrolling
        let bg_y = ((line as u8).wrapping_add(self.scy)) as usize;
        let tile_y = bg_y / 8; // Quelle ligne de tiles
        let pixel_y = bg_y % 8; // Quelle ligne dans le tile

        // Déterminer quelle map utiliser (LCDC bit 3)
        let bg_map_base = if (self.lcdc & 0x08) != 0 {
            0x9C00
        } else {
            0x9800
        };

        // Déterminer quelle zone de tiles utiliser (LCDC bit 4)
        let tile_data_base = if (self.lcdc & 0x10) != 0 {
            0x8000
        } else {
            0x8800
        };
        let signed_tiles = (self.lcdc & 0x10) == 0;

        for x in 0..160 {
            // Calculer la position du background avec scrolling
            let bg_x = ((x as u8).wrapping_add(self.scx)) as usize;
            let tile_x = bg_x / 8; // Quelle colonne de tiles
            let pixel_x = bg_x % 8; // Quelle colonne dans le tile

            // Lire l'ID du tile depuis la background map
            let tile_map_addr = bg_map_base + (tile_y * 32) + tile_x;
            let tile_id = memory.read_vram(tile_map_addr as u16);

            // Calculer l'adresse du tile dans la VRAM
            let tile_addr = if signed_tiles {
                // Mode signé: 0x9000 + (tile_id as i8 as i16) * 16
                (0x9000_u16).wrapping_add(((tile_id as i8 as i16) * 16) as u16)
            } else {
                // Mode non-signé: 0x8000 + tile_id * 16
                tile_data_base + (tile_id as u16 * 16)
            };

            // Lire les 2 bytes de la ligne du tile
            let line_addr = tile_addr + (pixel_y as u16 * 2);
            let byte1 = memory.read_vram(line_addr);
            let byte2 = memory.read_vram(line_addr + 1);

            // Extraire la couleur du pixel (2 bits)
            let bit_pos = 7 - pixel_x;
            let color_bit_0 = (byte1 >> bit_pos) & 1;
            let color_bit_1 = (byte2 >> bit_pos) & 1;
            let color_id = (color_bit_1 << 1) | color_bit_0;

            // Appliquer la palette background
            let final_color = self.apply_bg_palette(color_id);

            self.framebuffer[line][x] = final_color;
        }
    }

    fn apply_bg_palette(&self, color_id: u8) -> u8 {
        // Extraire la couleur depuis la palette BGP
        let shift = color_id * 2;
        (self.bgp >> shift) & 0x03
    }

    // Display current palette
    fn decode_palette(&self, palette: u8, name: &str) {
        let color0 = palette & 0x03;
        let color1 = (palette >> 2) & 0x03;
        let color2 = (palette >> 4) & 0x03;
        let color3 = (palette >> 6) & 0x03;

        let color_names = ["White", "Light Gray", "Dark Gray", "Black"];

        println!(
            "  {} palette: {} -> {} -> {} -> {}",
            name,
            color_names[color0 as usize],
            color_names[color1 as usize],
            color_names[color2 as usize],
            color_names[color3 as usize]
        );
    }

    pub fn print_screen(&self) {
        println!("\n┌{}┐", "─".repeat(160));

        for line in self.framebuffer.iter() {
            print!("│");
            for &pixel in line.iter() {
                let char = match pixel {
                    0 => ' ', // Blanc
                    1 => '░', // Gris clair
                    2 => '▒', // Gris foncé
                    3 => '█', // Noir
                    _ => '?',
                };
                print!("{}", char);
            }
            println!("│");
        }

        println!("└{}┘", "─".repeat(160));
    }

    pub fn print_screen_small(&self) {
        if !self.is_lcd_enabled() {
            return;
        }

        // Version réduite pour debug (80x36 au lieu de 160x144)
        println!("┌{}┐", "─".repeat(80));

        for y in (0..144).step_by(4) {
            print!("│");
            for x in (0..160).step_by(2) {
                let pixel = self.framebuffer[y][x];
                let char = match pixel {
                    0 => ' ', // Blanc
                    1 => '░', // Gris clair
                    2 => '▒', // Gris foncé
                    3 => '█', // Noir
                    _ => '?',
                };
                print!("{}", char);
            }
            println!("│");
        }

        println!("└{}┘", "─".repeat(80));
    }
}

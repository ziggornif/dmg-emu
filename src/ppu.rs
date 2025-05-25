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

    cycles: u32,
    mode: PPUMode,
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
            ly: 0x00,
            lyc: 0x00,
            wy: 0x00,
            wx: 0x00,
            cycles: 0,
            mode: PPUMode::OAMScan,
        }
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
            _ => {
                println!("Register not implemented: 0x{:04X}", address);
                0xFF
            }
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => {
                self.lcdc = value;

                if (value & 0x80) == 0 {
                    self.ly = 0;
                    self.cycles = 0;
                    self.mode = PPUMode::HBLank;
                }
            }
            0xFF41 => self.stat = (self.stat & 0x07) | (value & 0xF8), // (current & 0000 0111) | (new_val & 1111 1000)
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // readonly
            0xFF45 => self.lyc = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {
                println!(
                    "Register not implemented: 0x{:04X} = 0x{:02X}",
                    address, value
                );
            }
        }
    }

    pub fn step(&mut self, cpu_cycles: u8) -> bool {
        if (self.lcdc & 0x80) == 0 {
            return false;
        }

        self.cycles += cpu_cycles as u32;
        let mut vblank_interrupt = false;

        match self.mode {
            PPUMode::OAMScan => {
                if self.cycles >= 80 {
                    self.cycles -= 80;
                    self.mode = PPUMode::Drawing;
                }
            }
            PPUMode::Drawing => {
                if self.cycles >= 172 {
                    self.cycles -= 172;
                    self.mode = PPUMode::HBLank;
                }
            }
            PPUMode::HBLank => {
                if self.cycles >= 204 {
                    self.cycles -= 204;
                    self.ly += 1;

                    if self.ly >= 144 {
                        self.mode = PPUMode::VBlank;
                        vblank_interrupt = true;
                    } else {
                        self.mode = PPUMode::OAMScan;
                    }
                }
            }
            PPUMode::VBlank => {
                if self.cycles >= 456 {
                    self.cycles -= 456;
                    self.ly += 1;

                    if self.ly >= 154 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance_ppu_lines(ppu: &mut PPU, lines: u32) {
        let total_cycles = lines * 456;
        let mut remaining = total_cycles;

        while remaining > 0 {
            let step_size = std::cmp::min(remaining, 100) as u8;
            ppu.step(step_size);
            remaining -= step_size as u32;
        }
    }

    #[test]
    fn test_ppu_new() {
        let ppu = PPU::new();

        assert_eq!(ppu.lcdc, 0x91);
        assert_eq!(ppu.stat, 0x00);
        assert_eq!(ppu.ly, 0x00);
        assert_eq!(ppu.lyc, 0x00);
        assert_eq!(ppu.scy, 0x00);
        assert_eq!(ppu.scx, 0x00);
        assert_eq!(ppu.wy, 0x00);
        assert_eq!(ppu.wy, 0x00);
        assert_eq!(ppu.cycles, 0);
        assert_eq!(ppu.mode, PPUMode::OAMScan);

        assert!(ppu.is_lcd_enabled());
    }

    #[test]
    fn test_register_read_write() {
        let mut ppu = PPU::new();

        // LCDC
        ppu.write_register(0xFF40, 0x85);
        assert_eq!(ppu.read_register(0xFF40), 0x85);

        // SCY/SCX
        ppu.write_register(0xFF42, 0x10);
        ppu.write_register(0xFF43, 0x20);
        assert_eq!(ppu.read_register(0xFF42), 0x10);
        assert_eq!(ppu.read_register(0xFF43), 0x20);

        // LYC
        ppu.write_register(0xFF45, 0x50);
        assert_eq!(ppu.read_register(0xFF45), 0x50);

        // Test Window
        ppu.write_register(0xFF4A, 0x88);
        ppu.write_register(0xFF4B, 0x07);
        assert_eq!(ppu.read_register(0xFF4A), 0x88);
        assert_eq!(ppu.read_register(0xFF4B), 0x07);
    }

    #[test]
    fn test_ly_read_only() {
        let mut ppu = PPU::new();

        assert_eq!(ppu.read_register(0xFF44), 0x00);

        ppu.write_register(0xFF44, 0x99);

        assert_eq!(ppu.read_register(0xFF44), 0x00);
    }

    #[test]
    fn test_stat_register() {
        let mut ppu = PPU::new();

        let initial_stat = ppu.read_register(0xFF41);

        assert_eq!(initial_stat & 0x03, 0x02); // initial = 0x00 | OAMScan 0x02

        ppu.write_register(0xFF41, 0x48);

        let stat_after_write = ppu.read_register(0xFF41);

        assert_eq!(stat_after_write & 0xF8, 0x48);
        assert_eq!(stat_after_write & 0x03, 0x02);
    }

    #[test]
    fn test_lcd_disable() {
        let mut ppu = PPU::new();

        advance_ppu_lines(&mut ppu, 1);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 2);

        ppu.write_register(0xFF40, 0x00);

        assert_eq!(ppu.ly, 0);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 0);
        assert!(!ppu.is_lcd_enabled());

        let old_ly = ppu.ly;
        let vblank = ppu.step(255);

        assert_eq!(ppu.ly, old_ly);
        assert!(!vblank);
    }

    #[test]
    fn test_oam_scan_to_drawing() {
        let mut ppu = PPU::new();

        assert_eq!(ppu.read_register(0xFF41) & 0x03, 2); // OAMScan

        let vblank = ppu.step(79);

        assert_eq!(ppu.read_register(0xFF41) & 0x03, 2); // OAMScan
        assert!(!vblank);

        let vblank = ppu.step(1);

        assert_eq!(ppu.read_register(0xFF41) & 0x03, 3); // Drawing
        assert!(!vblank);
    }

    #[test]
    fn test_drawing_to_hblank() {
        let mut ppu = PPU::new();

        ppu.step(80);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 3); // Drawing

        let vblank = ppu.step(171);

        assert_eq!(ppu.read_register(0xFF41) & 0x03, 3); // Drawing
        assert!(!vblank);

        let vblank = ppu.step(1);

        assert_eq!(ppu.read_register(0xFF41) & 0x03, 0); // HBlank
        assert!(!vblank);
    }

    #[test]
    fn test_hblank_to_next_line() {
        let mut ppu = PPU::new();

        ppu.step(80); // OAMScan → Drawing
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 3); // Drawing

        ppu.step(172); // Drawing → HBlank  
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 0); // HBlank ✅
        assert_eq!(ppu.ly, 0);

        let vblank = ppu.step(204);

        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 2); // OAMScan
        assert!(!vblank);
    }

    #[test]
    fn test_not_implemented() {
        let mut ppu = PPU::new();

        ppu.write_register(0xFFFF, 0x10);
        assert_eq!(ppu.read_register(0xFFFF), 0xFF);
    }
}

#[cfg(test)]
mod tests {
    use dmg_emu::ppu::{PPU, PPUMode};

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
        assert_eq!(ppu.get_cycles(), 0);
        assert_eq!(ppu.get_mode(), PPUMode::OAMScan);

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
    fn test_vblank_trigger() {
        let mut ppu = PPU::new();
        let mut vblank_triggered = false;

        let mut total_cycles = 0u32;
        let target_vblank = 144 * 456;

        while total_cycles < target_vblank && !vblank_triggered {
            let cycles = match total_cycles % 60 {
                0..=10 => 4,   // LD r,r
                11..=25 => 8,  // LD r,n
                26..=35 => 12, // JR cc,r8
                36..=45 => 16, // LD (nn),A
                46..=55 => 20, // JR r8
                _ => 24,       // CALL nn
            };

            let vblank = ppu.step(cycles);
            if vblank {
                vblank_triggered = true;
            }

            total_cycles += cycles as u32;
        }

        assert!(vblank_triggered);
        assert_eq!(ppu.ly, 144);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 1); // V-Blank
    }

    #[test]
    fn test_lcd_disable_realistic() {
        let mut ppu = PPU::new();

        advance_ppu_lines(&mut ppu, 1);

        assert_eq!(ppu.ly, 1);
        assert!(ppu.read_register(0xFF41) & 0x03 != 0);

        ppu.write_register(0xFF40, 0x00);

        assert_eq!(ppu.ly, 0);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 0); // HBlank
        assert!(!ppu.is_lcd_enabled());

        let realistic_cycles = [4, 8, 12, 16, 20, 24];
        for i in 0..1000 {
            ppu.step(realistic_cycles[i % realistic_cycles.len()]);
        }
        assert_eq!(ppu.ly, 0);
    }

    #[test]
    fn test_complete_frame() {
        let mut ppu = PPU::new();

        let instructions = [
            ("NOP", 4),        // 4 cycles
            ("LD B,n", 8),     // 8 cycles
            ("ADD A,B", 4),    // 4 cycles
            ("JR NZ,r8", 12),  // 12 cycles (branch taken)
            ("LD (HL),A", 8),  // 8 cycles
            ("INC HL", 8),     // 8 cycles
            ("DEC B", 4),      // 4 cycles
            ("JR Z,r8", 8),    // 8 cycles (branch not taken)
            ("CALL nn", 24),   // 24 cycles
            ("RET", 16),       // 16 cycles
            ("LD A,(HL+)", 8), // 8 cycles
            ("CP n", 8),       // 8 cycles
        ];

        for i in 0..16000 {
            let (_, cycles) = instructions[i % instructions.len()];
            let vblank = ppu.step(cycles);

            if vblank {
                break;
            }
        }

        assert_eq!(ppu.ly, 144);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 1); // VBlank

        for i in 0..4560 {
            let (_, cycles) = instructions[i % instructions.len()];
            ppu.step(cycles);

            if ppu.ly == 0 {
                break;
            }
        }

        assert_eq!(ppu.ly, 0);
        assert_eq!(ppu.read_register(0xFF41) & 0x03, 2); // OAMScan
    }

    #[test]
    fn test_not_implemented() {
        let mut ppu = PPU::new();

        ppu.write_register(0xFFFF, 0x10);
        assert_eq!(ppu.read_register(0xFFFF), 0xFF);
    }
}

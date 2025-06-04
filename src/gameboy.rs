use log::debug;

use crate::{bus::Bus, cpu::CPU};

pub struct Gameboy {
    pub cpu: CPU,
    pub bus: Bus,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), String> {
        self.bus.load_rom(rom_data)
    }

    pub fn step(&mut self) -> bool {
        let opcode = self.bus.read_byte(self.cpu.pc);

        self.cpu.pc += 1;

        let cycles = self.cpu.execute_instruction(opcode, &mut self.bus);

        debug!("About to call PPU step with {} cycles", cycles);
        let vblank_interrupt = self.bus.ppu_step(cycles);
        debug!("PPU step done, LY is now: {}", self.bus.ppu.ly);

        if vblank_interrupt && self.cpu.interrupts_enabled() {
            self.handle_vblank_interrupt();
        }

        vblank_interrupt
    }

    fn handle_vblank_interrupt(&mut self) {
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        self.bus.write_byte(self.cpu.sp, (self.cpu.pc >> 8) as u8);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        self.bus.write_byte(self.cpu.sp, self.cpu.pc as u8);

        self.cpu.pc = 0x0040;

        self.cpu.disable_interrupts();
    }

    pub fn run_frame(&mut self) -> u32 {
        let mut cycles = 0;
        loop {
            let vblank = self.step();
            cycles += 1;

            if vblank {
                break;
            }

            //avoid infinite loop protection
            if cycles > 70000 {
                break;
            }
        }

        cycles
    }

    pub fn get_framebuffer(&self) -> &[[u8; 160]] {
        &self.bus.ppu.framebuffer
    }

    pub fn print_debug_screen(&self) {
        self.bus.ppu.print_screen_small();
    }

    pub fn print_cpu_state(&self) {
        println!("CPU State:");
        println!(
            "  A: 0x{:02X}  F: 0x{:02X}  AF: 0x{:04X}",
            self.cpu.a,
            self.cpu.f,
            self.cpu.af()
        );
        println!(
            "  B: 0x{:02X}  C: 0x{:02X}  BC: 0x{:04X}",
            self.cpu.b,
            self.cpu.c,
            self.cpu.bc()
        );
        println!(
            "  D: 0x{:02X}  E: 0x{:02X}  DE: 0x{:04X}",
            self.cpu.d,
            self.cpu.e,
            self.cpu.de()
        );
        println!(
            "  H: 0x{:02X}  L: 0x{:02X}  HL: 0x{:04X}",
            self.cpu.h,
            self.cpu.l,
            self.cpu.hl()
        );
        println!("  SP: 0x{:04X}  PC: 0x{:04X}", self.cpu.sp, self.cpu.pc);
        println!(
            "  Flags: Z:{} N:{} H:{} C:{}",
            if self.cpu.flag_z() { "1" } else { "0" },
            if self.cpu.flag_n() { "1" } else { "0" },
            if self.cpu.flag_h() { "1" } else { "0" },
            if self.cpu.flag_c() { "1" } else { "0" }
        );
        println!(
            "  IME: {}",
            if self.cpu.interrupts_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    pub fn print_ppu_state(&self) {
        println!("PPU State:");
        println!(
            "  LCDC: 0x{:02X}  STAT: 0x{:02X}",
            self.bus.ppu.lcdc, self.bus.ppu.stat
        );
        println!("  LY: {}  LYC: {}", self.bus.ppu.ly, self.bus.ppu.lyc);
        println!("  SCX: {}  SCY: {}", self.bus.ppu.scx, self.bus.ppu.scy);
        println!(
            "  Mode: {:?}  Cycles: {}",
            self.bus.ppu.get_mode(),
            self.bus.ppu.get_cycles()
        );
        println!("  LCD Enabled: {}", self.bus.ppu.is_lcd_enabled());
    }
}

impl Default for Gameboy {
    fn default() -> Self {
        Self::new()
    }
}

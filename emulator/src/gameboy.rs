use crate::joypad::JoypadButton;
use crate::{bus::Bus, cpu::CPU, debug, error, print_cpu_state, print_ppu_state};

#[derive(Debug, Clone)]
pub struct Gameboy {
    pub cpu: CPU,
    pub bus: Bus,
    pub last_pc: u16,
    pub pc_repeat_count: u32,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(),
            last_pc: 0,
            pc_repeat_count: 0,
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), String> {
        self.bus.load_rom(rom_data)
    }

    fn validate_pc(&self) {
        match self.cpu.pc {
            0x0000..=0x7FFF => {} // ROM - OK
            0x8000..=0x9FFF => {} // VRAM - OK
            0xA000..=0xBFFF => {} // External RAM - OK
            0xC000..=0xFDFF => {} // WRAM - OK
            0xFE00..=0xFE9F => {} // OAM - OK
            0xFF00..=0xFF7F => {} // I/O Registers - OK
            0xFF80..=0xFFFE => {} // HRAM - OK
            0xFFFF => {}          // IE Register - OK
            _ => {
                error!("ERROR: PC in invalid zone: 0x{:04X}", self.cpu.pc);
                print_cpu_state!(self.cpu);
                print_ppu_state!(self.bus.ppu);
                panic!("PC in invalid memory zone");
            }
        }
    }

    pub fn handle_input(&mut self, button: JoypadButton, pressed: bool) {
        self.bus.set_joypad_input(button, pressed);

        if pressed && self.bus.joypad_interrupt() {
            let if_reg = self.bus.read_byte(0xFF0F);
            self.bus.write_byte(0xFF0F, if_reg | 0x10);
        }
    }

    pub fn step(&mut self) -> bool {
        if self.cpu.pc == self.last_pc {
            self.pc_repeat_count += 1;
            if self.pc_repeat_count > 1000 {
                println!("\n=== INFINITE LOOP DETECTED ===");
                println!("PC at: 0x{:04X}", self.cpu.pc);
                return true;
            }
        } else {
            self.last_pc = self.cpu.pc;
            self.pc_repeat_count = 0;
        }

        if let Some(data) = self.get_serial_output() {
            let ch = data as char;
            if ch == '\n' || ch == ' ' {
                print!("\n");
            } else {
                print!("{}", ch);
            }
        }

        self.validate_pc();

        let opcode = self.bus.read_byte(self.cpu.pc);
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        let cycles = self.cpu.execute_instruction(opcode, &mut self.bus);

        let timer_interrupt = self.bus.timer_step(cycles);
        if timer_interrupt {
            let if_reg = self.bus.read_byte(0xFF0F);
            self.bus.write_byte(0xFF0F, if_reg | 0x04);
        }

        let vblank_interrupt = self.bus.ppu_step(cycles);

        self.handle_interrupts(vblank_interrupt, timer_interrupt);

        vblank_interrupt
    }

    fn handle_interrupts(&mut self, vblank: bool, timer: bool) {
        if !self.cpu.interrupts_enabled() {
            return;
        }

        let mut interrupt_triggered = false;

        // VBlank interrupt
        if vblank {
            let ie_reg = self.bus.read_byte(0xFFFF);
            if (ie_reg & 0x01) != 0 {
                debug!("🔥 Handling VBlank interrupt");
                self.handle_interrupt(0x40, 0x01);
                interrupt_triggered = true;
            }
        }

        // Timer interrupt
        if timer && !interrupt_triggered {
            let ie_reg = self.bus.read_byte(0xFFFF);
            if (ie_reg & 0x04) != 0 {
                debug!("🔥 Handling Timer interrupt");
                self.handle_interrupt(0x50, 0x04);
                interrupt_triggered = true;
            }
        }

        // Others
        if !interrupt_triggered {
            let if_reg = self.bus.read_byte(0xFF0F);
            let ie_reg = self.bus.read_byte(0xFFFF);
            let pending = if_reg & ie_reg;

            if pending & 0x02 != 0 {
                // LCD STAT interrupt
                debug!("🔥 Handling LCD STAT interrupt");
                self.handle_interrupt(0x48, 0x02);
            } else if pending & 0x08 != 0 {
                // Serial interrupt
                debug!("🔥 Handling Serial interrupt");
                self.handle_interrupt(0x58, 0x08);
            } else if pending & 0x10 != 0 {
                // Joypad interrupt
                debug!("🔥 Handling Joypad interrupt");
                self.handle_interrupt(0x60, 0x10);
            }
        }
    }

    fn handle_interrupt(&mut self, vector: u16, flag_bit: u8) {
        self.cpu.disable_interrupts();
        self.cpu.stack_push(&mut self.bus, self.cpu.pc);
        self.cpu.pc = vector;

        let if_reg = self.bus.read_byte(0xFF0F);
        self.bus.write_byte(0xFF0F, if_reg & !flag_bit);
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

    pub fn get_serial_output(&mut self) -> Option<u8> {
        if self.bus.read_byte(0xFF02) & 0x80 != 0 {
            let data = self.bus.read_byte(0xFF01);
            self.bus
                .write_byte(0xFF02, self.bus.read_byte(0xFF02) & 0x7F);
            Some(data)
        } else {
            None
        }
    }

    pub fn print_debug_screen(&self) {
        self.bus.ppu.print_screen();
    }
}

impl Default for Gameboy {
    fn default() -> Self {
        Self::new()
    }
}

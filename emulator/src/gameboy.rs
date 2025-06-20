use crate::{bus::Bus, cpu::CPU, error, info, print_cpu_state, print_ppu_state};

#[derive(Debug, Clone)]
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

    pub fn step(&mut self) -> bool {
        if self.cpu.pc == 0x0100 {
            info!("Back to program start - possible infinite loop");
        }

        if let Some(data) = self.get_serial_output() {
            info!("{}", data as char);
        }

        self.validate_pc();

        let opcode = self.bus.read_byte(self.cpu.pc);
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        let cycles = self.cpu.execute_instruction(opcode, &mut self.bus);

        let vblank_interrupt = self.bus.ppu_step(cycles);

        if vblank_interrupt && self.cpu.interrupts_enabled() {
            self.cpu.wake_from_halt();
            self.handle_vblank_interrupt();
        }

        vblank_interrupt
    }

    fn handle_vblank_interrupt(&mut self) {
        self.cpu.stack_push(&mut self.bus, self.cpu.pc);
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

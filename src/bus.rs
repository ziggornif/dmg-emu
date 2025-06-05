use crate::{memory::Memory, ppu::PPU};

pub struct Bus {
    pub memory: Memory,
    pub ppu: PPU,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            ppu: PPU::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF40..=0xFF4B => self.ppu.read_register(address),
            0x8000..=0x9FFF => {
                if self.ppu.can_access_vram() {
                    self.memory.read_byte(address)
                } else {
                    0xFF
                }
            }
            0xFE00..=0xFE9F => {
                if self.ppu.can_access_oam() {
                    self.memory.read_byte(address)
                } else {
                    0xFF
                }
            }
            _ => self.memory.read_byte(address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF40..=0xFF4B => self.ppu.write_register(address, value),
            0x8000..=0x9FFF => {
                if self.ppu.can_access_vram() {
                    self.memory.write_byte(address, value)
                }
            }
            0xFE00..=0xFE9F => {
                if self.ppu.can_access_oam() {
                    self.memory.write_byte(address, value)
                }
            }
            _ => self.memory.write_byte(address, value),
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), String> {
        self.memory.load_rom(rom_data)
    }

    pub fn ppu_step(&mut self, cpu_cycles: u8) -> bool {
        self.ppu.step(cpu_cycles, &self.memory)
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

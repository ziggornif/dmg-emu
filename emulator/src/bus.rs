use crate::{memory::Memory, ppu::PPU};

#[derive(Debug, Clone)]
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
            0xFF46 => 0xFF, // DMA register is always 0xFF
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
            0xFF46 => {
                // DMA Transfer
                self.perform_dma_transfer(value);
            }
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

    fn perform_dma_transfer(&mut self, source_high_byte: u8) {
        let source_address = (source_high_byte as u16) << 8;

        for i in 0..0xA0 {
            let source_addr = source_address + i;
            let dest_addr = 0xFE00 + i;

            let data = self.memory.read_byte(source_addr);

            self.memory.write_byte(dest_addr, data);
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, value as u8);
        self.write_byte(address.wrapping_add(1), (value >> 8) as u8);
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

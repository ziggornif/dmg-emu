use crate::ppu::PPU;

pub struct Memory {
    data: [u8; 0x10000],
    pub ppu: PPU,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: [0; 0x10000],
            ppu: PPU::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF40..=0xFF4B => self.ppu.read_register(address),
            _ => self.data[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF40..=0xFF4B => self.ppu.write_register(address, value),
            _ => self.data[address as usize] = value,
        }
    }

    pub fn step_ppu(&mut self, cpu_cycles: u8) -> bool {
        self.ppu.step(cpu_cycles)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

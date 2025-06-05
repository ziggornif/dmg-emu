pub struct Memory {
    data: [u8; 0x10000],
    vram: [u8; 0x2000],
    oam: [u8; 0xA0],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: [0xFF; 0x10000],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => {
                let vram_addr = (address - 0x8000) as usize;
                self.vram[vram_addr]
            }
            0xFE00..=0xFE9F => {
                let oam_addr = (address - 0xFE00) as usize;
                self.oam[oam_addr]
            }
            _ => self.data[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                let vram_addr = (address - 0x8000) as usize;
                self.vram[vram_addr] = value;
            }
            0xFE00..=0xFE9F => {
                let oam_addr = (address - 0xFE00) as usize;
                self.oam[oam_addr] = value;
            }
            _ => self.data[address as usize] = value,
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => {
                let vram_addr = (address - 0x8000) as usize;
                self.vram[vram_addr]
            }
            _ => 0xFF,
        }
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        match address {
            0xFE00..=0xFE9F => {
                let oam_addr = (address - 0xFE00) as usize;
                self.oam[oam_addr]
            }
            _ => 0xFF,
        }
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), String> {
        if rom_data.is_empty() {
            return Err("ROM is empty".to_string());
        }

        let copy_size = std::cmp::min(rom_data.len(), 0x8000);
        self.data[0x0000..0x8000].fill(0xFF);
        self.data[0x0000..copy_size].copy_from_slice(&rom_data[0..copy_size]);

        println!("ROM loaded successfully: {} bytes", copy_size);

        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

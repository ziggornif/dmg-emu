#[derive(Debug, Clone)]
pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,

    div_counter: u16,
    tima_counter: u16,

    pub interrupt_requested: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            div_counter: 0,
            tima_counter: 0,
            interrupt_requested: false,
        }
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                self.div = 0;
                self.div_counter = 0;
            }
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value & 0x07,
            _ => {}
        }
    }

    pub fn step(&mut self, cpu_cycles: u8) {
        let timer_cycles = cpu_cycles as u16 * 4;

        self.div_counter = self.div_counter.wrapping_add(timer_cycles);
        while self.div_counter >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_counter = self.div_counter.wrapping_sub(256);
        }

        if self.is_timer_enabled() {
            let tima_increment_rate = self.get_tima_frequency();

            self.tima_counter = self.tima_counter.wrapping_add(timer_cycles);

            while self.tima_counter >= tima_increment_rate {
                self.tima_counter = self.tima_counter.wrapping_sub(tima_increment_rate);

                // Increment TIMA
                if self.tima == 0xFF {
                    self.tima = self.tma;
                    self.interrupt_requested = true;
                } else {
                    self.tima = self.tima.wrapping_add(1);
                }
            }
        }
    }

    fn is_timer_enabled(&self) -> bool {
        (self.tac & 0x04) != 0
    }

    fn get_tima_frequency(&self) -> u16 {
        match self.tac & 0x03 {
            0 => 1024, // 4096 Hz   (4194304 / 4096 = 1024 cycles)
            1 => 16,   // 262144 Hz (4194304 / 262144 = 16 cycles)
            2 => 64,   // 65536 Hz  (4194304 / 65536 = 64 cycles)
            3 => 256,  // 16384 Hz  (4194304 / 16384 = 256 cycles)
            _ => unreachable!(),
        }
    }

    pub fn take_interrupt(&mut self) -> bool {
        let interrupt = self.interrupt_requested;
        self.interrupt_requested = false;
        interrupt
    }
}

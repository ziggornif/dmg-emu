const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

// Read masks: bits that return 1 on read (write-only bits)
const NR10_READ_MASK: u8 = 0x80;
const NR11_READ_MASK: u8 = 0x3F; // only duty bits readable
const NR12_READ_MASK: u8 = 0x00;
const NR13_READ_MASK: u8 = 0xFF; // write-only
const NR14_READ_MASK: u8 = 0xBF; // bit 6 readable, rest masked
const NR21_READ_MASK: u8 = 0x3F;
const NR22_READ_MASK: u8 = 0x00;
const NR23_READ_MASK: u8 = 0xFF;
const NR24_READ_MASK: u8 = 0xBF;
const NR30_READ_MASK: u8 = 0x7F;
const NR31_READ_MASK: u8 = 0xFF;
const NR32_READ_MASK: u8 = 0x9F;
const NR33_READ_MASK: u8 = 0xFF;
const NR34_READ_MASK: u8 = 0xBF;
const NR41_READ_MASK: u8 = 0xFF;
const NR42_READ_MASK: u8 = 0x00;
const NR43_READ_MASK: u8 = 0x00;
const NR44_READ_MASK: u8 = 0xBF;

const CPU_CLOCK: u32 = 4_194_304;
const FRAME_SEQUENCER_RATE: u16 = 8192; // CPU clocks per frame sequencer tick

#[derive(Debug, Clone)]
pub struct SquareChannel {
    pub enabled: bool,

    // Sweep (channel 1 only)
    pub has_sweep: bool,
    sweep_pace: u8,
    sweep_direction: bool, // false = add, true = subtract
    sweep_step: u8,
    sweep_timer: u8,
    sweep_shadow_freq: u16,
    sweep_enabled: bool,

    // Duty
    duty: u8,
    duty_position: u8,

    // Length
    length_counter: u16,
    length_enabled: bool,

    // Volume envelope
    initial_volume: u8,
    envelope_direction: bool, // false = down, true = up
    envelope_pace: u8,
    envelope_timer: u8,
    current_volume: u8,

    // Frequency
    frequency: u16,
    frequency_timer: u16,

    // Raw registers
    pub nr0: u8, // NR10 (ch1) or unused (ch2)
    pub nr1: u8, // NRx1
    pub nr2: u8, // NRx2
    pub nr3: u8, // NRx3
    pub nr4: u8, // NRx4
}

impl SquareChannel {
    pub fn new(has_sweep: bool) -> Self {
        Self {
            enabled: false,
            has_sweep,
            sweep_pace: 0,
            sweep_direction: false,
            sweep_step: 0,
            sweep_timer: 0,
            sweep_shadow_freq: 0,
            sweep_enabled: false,
            duty: 0,
            duty_position: 0,
            length_counter: 0,
            length_enabled: false,
            initial_volume: 0,
            envelope_direction: false,
            envelope_pace: 0,
            envelope_timer: 0,
            current_volume: 0,
            frequency: 0,
            frequency_timer: 0,
            nr0: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
        }
    }

    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => {
                // NRx0 (sweep, channel 1 only)
                self.nr0 = value;
                if self.has_sweep {
                    self.sweep_pace = (value >> 4) & 0x07;
                    self.sweep_direction = (value & 0x08) != 0;
                    self.sweep_step = value & 0x07;
                }
            }
            1 => {
                // NRx1 (duty + length)
                self.nr1 = value;
                self.duty = (value >> 6) & 0x03;
                self.length_counter = 64 - (value & 0x3F) as u16;
            }
            2 => {
                // NRx2 (volume envelope)
                self.nr2 = value;
                self.initial_volume = (value >> 4) & 0x0F;
                self.envelope_direction = (value & 0x08) != 0;
                self.envelope_pace = value & 0x07;

                // DAC disabled if top 5 bits are 0
                if value & 0xF8 == 0 {
                    self.enabled = false;
                }
            }
            3 => {
                // NRx3 (frequency low)
                self.nr3 = value;
                self.frequency = (self.frequency & 0x700) | value as u16;
            }
            4 => {
                // NRx4 (trigger + length enable + frequency high)
                self.nr4 = value;
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);
                self.length_enabled = (value & 0x40) != 0;

                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.frequency_timer = (2048 - self.frequency) * 4;
        self.current_volume = self.initial_volume;
        self.envelope_timer = self.envelope_pace;

        // DAC check
        if self.nr2 & 0xF8 == 0 {
            self.enabled = false;
        }

        // Sweep init (channel 1)
        if self.has_sweep {
            self.sweep_shadow_freq = self.frequency;
            self.sweep_timer = if self.sweep_pace != 0 {
                self.sweep_pace
            } else {
                8
            };
            self.sweep_enabled = self.sweep_pace != 0 || self.sweep_step != 0;

            if self.sweep_step != 0 {
                let new_freq = self.calculate_sweep_frequency();
                if new_freq > 2047 {
                    self.enabled = false;
                }
            }
        }
    }

    fn calculate_sweep_frequency(&self) -> u16 {
        let shifted = self.sweep_shadow_freq >> self.sweep_step;
        if self.sweep_direction {
            self.sweep_shadow_freq.wrapping_sub(shifted)
        } else {
            self.sweep_shadow_freq.wrapping_add(shifted)
        }
    }

    pub fn step_frequency(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 4;
            self.duty_position = (self.duty_position + 1) % 8;
        }
    }

    pub fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn step_envelope(&mut self) {
        if self.envelope_pace == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_pace;

            if self.envelope_direction && self.current_volume < 15 {
                self.current_volume += 1;
            } else if !self.envelope_direction && self.current_volume > 0 {
                self.current_volume -= 1;
            }
        }
    }

    pub fn step_sweep(&mut self) {
        if !self.has_sweep {
            return;
        }

        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }

        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_pace != 0 {
                self.sweep_pace
            } else {
                8
            };

            if self.sweep_enabled && self.sweep_pace != 0 {
                let new_freq = self.calculate_sweep_frequency();
                if new_freq > 2047 {
                    self.enabled = false;
                } else if self.sweep_step != 0 {
                    self.sweep_shadow_freq = new_freq;
                    self.frequency = new_freq;

                    // Overflow check again
                    if self.calculate_sweep_frequency() > 2047 {
                        self.enabled = false;
                    }
                }
            }
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let sample = DUTY_TABLE[self.duty as usize][self.duty_position as usize];
        (sample as f32) * (self.current_volume as f32 / 15.0)
    }

    pub fn reset(&mut self) {
        let has_sweep = self.has_sweep;
        *self = Self::new(has_sweep);
    }
}

#[derive(Debug, Clone)]
pub struct WaveChannel {
    pub enabled: bool,

    // Length
    length_counter: u16,
    length_enabled: bool,

    // Volume
    volume_code: u8,

    // Frequency
    frequency: u16,
    frequency_timer: u16,

    // Wave RAM
    pub wave_ram: [u8; 16],
    wave_position: u8,

    // DAC
    dac_enabled: bool,

    // Raw registers
    pub nr0: u8, // NR30
    pub nr1: u8, // NR31
    pub nr2: u8, // NR32
    pub nr3: u8, // NR33
    pub nr4: u8, // NR34
}

impl Default for WaveChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            length_enabled: false,
            volume_code: 0,
            frequency: 0,
            frequency_timer: 0,
            wave_ram: [0; 16],
            wave_position: 0,
            dac_enabled: false,
            nr0: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
        }
    }

    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => {
                // NR30 - DAC enable
                self.nr0 = value;
                self.dac_enabled = (value & 0x80) != 0;
                if !self.dac_enabled {
                    self.enabled = false;
                }
            }
            1 => {
                // NR31 - Length
                self.nr1 = value;
                self.length_counter = 256 - value as u16;
            }
            2 => {
                // NR32 - Volume code
                self.nr2 = value;
                self.volume_code = (value >> 5) & 0x03;
            }
            3 => {
                // NR33 - Frequency low
                self.nr3 = value;
                self.frequency = (self.frequency & 0x700) | value as u16;
            }
            4 => {
                // NR34 - Trigger + length enable + frequency high
                self.nr4 = value;
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);
                self.length_enabled = (value & 0x40) != 0;

                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 256;
        }

        self.frequency_timer = (2048 - self.frequency) * 2;
        self.wave_position = 0;

        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn step_frequency(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 2;
            self.wave_position = (self.wave_position + 1) % 32;
        }
    }

    pub fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let byte_index = (self.wave_position / 2) as usize;
        let sample = if self.wave_position.is_multiple_of(2) {
            (self.wave_ram[byte_index] >> 4) & 0x0F
        } else {
            self.wave_ram[byte_index] & 0x0F
        };

        let shifted = match self.volume_code {
            0 => sample >> 4, // mute (shift by 4 = 0)
            1 => sample,      // 100%
            2 => sample >> 1, // 50%
            3 => sample >> 2, // 25%
            _ => 0,
        };

        shifted as f32 / 15.0
    }

    pub fn reset(&mut self) {
        let wave_ram = self.wave_ram;
        *self = Self::new();
        self.wave_ram = wave_ram; // Wave RAM persists across APU power off
    }
}

#[derive(Debug, Clone)]
pub struct NoiseChannel {
    pub enabled: bool,

    // Length
    length_counter: u16,
    length_enabled: bool,

    // Volume envelope
    initial_volume: u8,
    envelope_direction: bool,
    envelope_pace: u8,
    envelope_timer: u8,
    current_volume: u8,

    // Noise
    clock_shift: u8,
    width_mode: bool, // false = 15-bit, true = 7-bit
    divisor_code: u8,
    lfsr: u16,
    frequency_timer: u32,

    // Raw registers
    pub nr1: u8, // NR41
    pub nr2: u8, // NR42
    pub nr3: u8, // NR43
    pub nr4: u8, // NR44
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            length_enabled: false,
            initial_volume: 0,
            envelope_direction: false,
            envelope_pace: 0,
            envelope_timer: 0,
            current_volume: 0,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            lfsr: 0x7FFF,
            frequency_timer: 0,
            nr1: 0,
            nr2: 0,
            nr3: 0,
            nr4: 0,
        }
    }

    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            1 => {
                // NR41 - Length
                self.nr1 = value;
                self.length_counter = 64 - (value & 0x3F) as u16;
            }
            2 => {
                // NR42 - Volume envelope
                self.nr2 = value;
                self.initial_volume = (value >> 4) & 0x0F;
                self.envelope_direction = (value & 0x08) != 0;
                self.envelope_pace = value & 0x07;

                if value & 0xF8 == 0 {
                    self.enabled = false;
                }
            }
            3 => {
                // NR43 - Polynomial counter
                self.nr3 = value;
                self.clock_shift = (value >> 4) & 0x0F;
                self.width_mode = (value & 0x08) != 0;
                self.divisor_code = value & 0x07;
            }
            4 => {
                // NR44 - Trigger + length enable
                self.nr4 = value;
                self.length_enabled = (value & 0x40) != 0;

                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.frequency_timer = self.get_frequency_timer();
        self.lfsr = 0x7FFF;
        self.current_volume = self.initial_volume;
        self.envelope_timer = self.envelope_pace;

        if self.nr2 & 0xF8 == 0 {
            self.enabled = false;
        }
    }

    fn get_frequency_timer(&self) -> u32 {
        let divisor: u32 = match self.divisor_code {
            0 => 8,
            n => (n as u32) * 16,
        };
        divisor << self.clock_shift
    }

    pub fn step_frequency(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        if self.frequency_timer == 0 {
            self.frequency_timer = self.get_frequency_timer();

            let xor_result = (self.lfsr & 0x01) ^ ((self.lfsr >> 1) & 0x01);
            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

            if self.width_mode {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }
        }
    }

    pub fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn step_envelope(&mut self) {
        if self.envelope_pace == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_pace;

            if self.envelope_direction && self.current_volume < 15 {
                self.current_volume += 1;
            } else if !self.envelope_direction && self.current_volume > 0 {
                self.current_volume -= 1;
            }
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        // LFSR bit 0 inverted
        let sample = if self.lfsr & 0x01 == 0 { 1.0 } else { 0.0 };
        sample * (self.current_volume as f32 / 15.0)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[derive(Debug, Clone)]
pub struct APU {
    // Control registers
    pub nr50: u8, // 0xFF24 - Volume / VIN
    pub nr51: u8, // 0xFF25 - Panning
    pub nr52: u8, // 0xFF26 - Power control

    // 4 channels
    pub channel1: SquareChannel,
    pub channel2: SquareChannel,
    pub channel3: WaveChannel,
    pub channel4: NoiseChannel,

    // Frame sequencer (512 Hz = CPU_CLOCK / 8192)
    frame_sequencer_counter: u16,
    frame_sequencer_step: u8, // 0..7

    // Sample generation
    sample_counter: u32,
    sample_rate: u32,
    sample_buffer: Vec<(f32, f32)>,
}

impl Default for APU {
    fn default() -> Self {
        Self::new()
    }
}

impl APU {
    pub fn new() -> Self {
        Self {
            nr50: 0,
            nr51: 0,
            nr52: 0,
            channel1: SquareChannel::new(true),
            channel2: SquareChannel::new(false),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            frame_sequencer_counter: 0,
            frame_sequencer_step: 0,
            sample_counter: 0,
            sample_rate: 44100,
            sample_buffer: Vec::new(),
        }
    }

    fn is_powered(&self) -> bool {
        self.nr52 & 0x80 != 0
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            // Channel 1 registers
            0xFF10 => self.channel1.nr0 | NR10_READ_MASK,
            0xFF11 => self.channel1.nr1 | NR11_READ_MASK,
            0xFF12 => self.channel1.nr2 | NR12_READ_MASK,
            0xFF13 => NR13_READ_MASK, // write-only
            0xFF14 => self.channel1.nr4 | NR14_READ_MASK,

            // Channel 2 registers
            0xFF15 => 0xFF, // NR20 doesn't exist
            0xFF16 => self.channel2.nr1 | NR21_READ_MASK,
            0xFF17 => self.channel2.nr2 | NR22_READ_MASK,
            0xFF18 => NR23_READ_MASK, // write-only
            0xFF19 => self.channel2.nr4 | NR24_READ_MASK,

            // Channel 3 registers
            0xFF1A => self.channel3.nr0 | NR30_READ_MASK,
            0xFF1B => NR31_READ_MASK, // write-only
            0xFF1C => self.channel3.nr2 | NR32_READ_MASK,
            0xFF1D => NR33_READ_MASK, // write-only
            0xFF1E => self.channel3.nr4 | NR34_READ_MASK,

            // Channel 4 registers
            0xFF1F => 0xFF,           // unused
            0xFF20 => NR41_READ_MASK, // write-only
            0xFF21 => self.channel4.nr2 | NR42_READ_MASK,
            0xFF22 => self.channel4.nr3 | NR43_READ_MASK,
            0xFF23 => self.channel4.nr4 | NR44_READ_MASK,

            // Control registers
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let mut value = self.nr52 & 0x80;
                value |= 0x70; // bits 4-6 always read as 1
                if self.channel1.enabled {
                    value |= 0x01;
                }
                if self.channel2.enabled {
                    value |= 0x02;
                }
                if self.channel3.enabled {
                    value |= 0x04;
                }
                if self.channel4.enabled {
                    value |= 0x08;
                }
                value
            }

            // Wave RAM
            0xFF30..=0xFF3F => {
                let index = (address - 0xFF30) as usize;
                self.channel3.wave_ram[index]
            }

            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        // Wave RAM is always writable
        if (0xFF30..=0xFF3F).contains(&address) {
            let index = (address - 0xFF30) as usize;
            self.channel3.wave_ram[index] = value;
            return;
        }

        // NR52 is always writable
        if address == 0xFF26 {
            let was_on = self.is_powered();
            self.nr52 = (self.nr52 & 0x7F) | (value & 0x80);

            if was_on && !self.is_powered() {
                // Power off: reset all registers
                self.channel1.reset();
                self.channel2.reset();
                self.channel3.reset();
                self.channel4.reset();
                self.nr50 = 0;
                self.nr51 = 0;
            }
            return;
        }

        // Block writes when APU is off
        if !self.is_powered() {
            // Exception: length counters (NRx1) can be written when off on DMG
            match address {
                0xFF11 => self.channel1.write_register(1, value),
                0xFF16 => self.channel2.write_register(1, value),
                0xFF1B => self.channel3.write_register(1, value),
                0xFF20 => self.channel4.write_register(1, value),
                _ => {}
            }
            return;
        }

        match address {
            // Channel 1
            0xFF10 => self.channel1.write_register(0, value),
            0xFF11 => self.channel1.write_register(1, value),
            0xFF12 => self.channel1.write_register(2, value),
            0xFF13 => self.channel1.write_register(3, value),
            0xFF14 => self.channel1.write_register(4, value),

            // Channel 2
            0xFF16 => self.channel2.write_register(1, value),
            0xFF17 => self.channel2.write_register(2, value),
            0xFF18 => self.channel2.write_register(3, value),
            0xFF19 => self.channel2.write_register(4, value),

            // Channel 3
            0xFF1A => self.channel3.write_register(0, value),
            0xFF1B => self.channel3.write_register(1, value),
            0xFF1C => self.channel3.write_register(2, value),
            0xFF1D => self.channel3.write_register(3, value),
            0xFF1E => self.channel3.write_register(4, value),

            // Channel 4
            0xFF20 => self.channel4.write_register(1, value),
            0xFF21 => self.channel4.write_register(2, value),
            0xFF22 => self.channel4.write_register(3, value),
            0xFF23 => self.channel4.write_register(4, value),

            // Control
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,

            _ => {}
        }
    }

    pub fn step(&mut self, cpu_cycles: u8) {
        if !self.is_powered() {
            return;
        }

        let t_cycles = cpu_cycles as u32 * 4;

        for _ in 0..t_cycles {
            // Tick frequency timers every T-cycle
            self.channel1.step_frequency();
            self.channel2.step_frequency();
            self.channel3.step_frequency();
            self.channel4.step_frequency();

            // Frame sequencer
            self.frame_sequencer_counter += 1;
            if self.frame_sequencer_counter >= FRAME_SEQUENCER_RATE {
                self.frame_sequencer_counter = 0;
                self.clock_frame_sequencer();
            }

            // Sample generation
            self.sample_counter += self.sample_rate;
            if self.sample_counter >= CPU_CLOCK {
                self.sample_counter -= CPU_CLOCK;
                let sample = self.mix_samples();
                self.sample_buffer.push(sample);
            }
        }
    }

    fn clock_frame_sequencer(&mut self) {
        match self.frame_sequencer_step {
            0 => {
                self.clock_length();
            }
            1 => {}
            2 => {
                self.clock_length();
                self.clock_sweep();
            }
            3 => {}
            4 => {
                self.clock_length();
            }
            5 => {}
            6 => {
                self.clock_length();
                self.clock_sweep();
            }
            7 => {
                self.clock_envelope();
            }
            _ => {}
        }
        self.frame_sequencer_step = (self.frame_sequencer_step + 1) % 8;
    }

    fn clock_length(&mut self) {
        self.channel1.step_length();
        self.channel2.step_length();
        self.channel3.step_length();
        self.channel4.step_length();
    }

    fn clock_sweep(&mut self) {
        self.channel1.step_sweep();
    }

    fn clock_envelope(&mut self) {
        self.channel1.step_envelope();
        self.channel2.step_envelope();
        self.channel4.step_envelope();
    }

    fn mix_samples(&self) -> (f32, f32) {
        let ch1 = self.channel1.output();
        let ch2 = self.channel2.output();
        let ch3 = self.channel3.output();
        let ch4 = self.channel4.output();

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // NR51 panning
        if self.nr51 & 0x10 != 0 {
            left += ch1;
        }
        if self.nr51 & 0x20 != 0 {
            left += ch2;
        }
        if self.nr51 & 0x40 != 0 {
            left += ch3;
        }
        if self.nr51 & 0x80 != 0 {
            left += ch4;
        }

        if self.nr51 & 0x01 != 0 {
            right += ch1;
        }
        if self.nr51 & 0x02 != 0 {
            right += ch2;
        }
        if self.nr51 & 0x04 != 0 {
            right += ch3;
        }
        if self.nr51 & 0x08 != 0 {
            right += ch4;
        }

        // NR50 volume (0-7)
        let left_vol = ((self.nr50 >> 4) & 0x07) as f32 + 1.0;
        let right_vol = (self.nr50 & 0x07) as f32 + 1.0;

        left = left * left_vol / 32.0;
        right = right * right_vol / 32.0;

        (left, right)
    }

    pub fn take_samples(&mut self) -> Vec<(f32, f32)> {
        std::mem::take(&mut self.sample_buffer)
    }
}

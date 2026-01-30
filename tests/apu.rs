#[cfg(test)]
mod tests {
    use emulator::apu::APU;

    #[test]
    fn test_apu_new() {
        let apu = APU::new();

        assert_eq!(apu.nr50, 0);
        assert_eq!(apu.nr51, 0);
        assert_eq!(apu.nr52, 0);
        assert!(!apu.channel1.enabled);
        assert!(!apu.channel2.enabled);
        assert!(!apu.channel3.enabled);
        assert!(!apu.channel4.enabled);

        // NR52 should read as 0x70 (bits 4-6 always 1, power off, no channels)
        assert_eq!(apu.read_register(0xFF26), 0x70);
    }

    #[test]
    fn test_nr52_power_control() {
        let mut apu = APU::new();

        // Power on
        apu.write_register(0xFF26, 0x80);
        assert_eq!(apu.read_register(0xFF26) & 0x80, 0x80);

        // Write some registers while on
        apu.write_register(0xFF24, 0x77);
        apu.write_register(0xFF25, 0xFF);
        assert_eq!(apu.nr50, 0x77);
        assert_eq!(apu.nr51, 0xFF);

        // Power off - should reset registers
        apu.write_register(0xFF26, 0x00);
        assert_eq!(apu.read_register(0xFF26) & 0x80, 0x00);
        assert_eq!(apu.nr50, 0x00);
        assert_eq!(apu.nr51, 0x00);
    }

    #[test]
    fn test_wave_ram_read_write() {
        let mut apu = APU::new();

        // Wave RAM is always writable, even when APU is off
        for i in 0..16u8 {
            apu.write_register(0xFF30 + i as u16, i * 0x11);
        }

        for i in 0..16u8 {
            assert_eq!(apu.read_register(0xFF30 + i as u16), i * 0x11);
        }

        // Wave RAM survives power cycle
        apu.write_register(0xFF26, 0x80); // on
        apu.write_register(0xFF26, 0x00); // off
        for i in 0..16u8 {
            assert_eq!(apu.read_register(0xFF30 + i as u16), i * 0x11);
        }
    }

    #[test]
    fn test_channel1_trigger() {
        let mut apu = APU::new();

        // Power on
        apu.write_register(0xFF26, 0x80);

        // Configure channel 1: volume, frequency, trigger
        apu.write_register(0xFF12, 0xF0); // NR12: volume 15, no envelope
        apu.write_register(0xFF13, 0x00); // NR13: freq low
        apu.write_register(0xFF14, 0x80); // NR14: trigger

        assert!(apu.channel1.enabled);
        // NR52 bit 0 should be set
        assert_eq!(apu.read_register(0xFF26) & 0x01, 0x01);
    }

    #[test]
    fn test_length_counter_disables_channel() {
        let mut apu = APU::new();

        // Power on
        apu.write_register(0xFF26, 0x80);

        // Channel 1: short length, enable length counter
        apu.write_register(0xFF11, 0x3F); // NR11: duty 0, length = 64 - 63 = 1
        apu.write_register(0xFF12, 0xF0); // NR12: volume 15
        apu.write_register(0xFF13, 0x00); // NR13: freq low
        apu.write_register(0xFF14, 0xC0); // NR14: trigger + length enable

        assert!(apu.channel1.enabled);

        // Step enough to trigger frame sequencer step 0 (length counter)
        // Frame sequencer ticks every 8192 T-cycles
        apu.step(255); // 255 * 4 = 1020 T-cycles
        for _ in 0..8 {
            apu.step(255); // 8 * 1020 = 8160 more
        }

        // After enough steps, the length counter should have expired
        // (length was 1, one length clock sets it to 0 and disables channel)
        assert!(!apu.channel1.enabled);
    }

    #[test]
    fn test_register_read_masks() {
        let apu = APU::new();

        // NR13 (0xFF13) is write-only, should read as 0xFF
        assert_eq!(apu.read_register(0xFF13), 0xFF);

        // NR23 (0xFF18) is write-only, should read as 0xFF
        assert_eq!(apu.read_register(0xFF18), 0xFF);

        // NR33 (0xFF1D) is write-only, should read as 0xFF
        assert_eq!(apu.read_register(0xFF1D), 0xFF);

        // NR14 (0xFF14) bit 6 readable, rest masked → should read with 0xBF mask
        assert_eq!(apu.read_register(0xFF14) & 0xBF, 0xBF);

        // NR10 (0xFF10) upper bit masked
        assert_eq!(apu.read_register(0xFF10) & 0x80, 0x80);

        // Unused register 0xFF15 should read 0xFF
        assert_eq!(apu.read_register(0xFF15), 0xFF);

        // Unused register 0xFF1F should read 0xFF
        assert_eq!(apu.read_register(0xFF1F), 0xFF);
    }

    #[test]
    fn test_apu_off_blocks_writes() {
        let mut apu = APU::new();

        // APU is off by default (NR52 bit 7 = 0)
        assert_eq!(apu.read_register(0xFF26) & 0x80, 0x00);

        // Writes to NR50/NR51 should be blocked when off
        apu.write_register(0xFF24, 0x77);
        assert_eq!(apu.nr50, 0x00);

        apu.write_register(0xFF25, 0xFF);
        assert_eq!(apu.nr51, 0x00);

        // NR52 write should always work
        apu.write_register(0xFF26, 0x80);
        assert_eq!(apu.read_register(0xFF26) & 0x80, 0x80);

        // Wave RAM should always be writable
        apu.write_register(0xFF26, 0x00); // turn off again
        apu.write_register(0xFF30, 0xAB);
        assert_eq!(apu.read_register(0xFF30), 0xAB);

        // Length counters (NRx1) can be written when off on DMG
        apu.write_register(0xFF11, 0x80); // NR11
        assert_eq!(apu.channel1.nr1, 0x80);
    }

    #[test]
    fn test_noise_channel_lfsr() {
        let mut apu = APU::new();

        // Power on
        apu.write_register(0xFF26, 0x80);

        // Configure noise channel
        apu.write_register(0xFF21, 0xF0); // NR42: volume 15
        apu.write_register(0xFF22, 0x00); // NR43: divisor 0, shift 0, 15-bit
        apu.write_register(0xFF23, 0x80); // NR44: trigger

        assert!(apu.channel4.enabled);

        // After trigger, LFSR should be 0x7FFF
        // (internal state, but we can check channel is producing output)
        // Step a bit to get some noise output
        apu.step(4);

        // Channel should still be enabled (no length counter active)
        assert!(apu.channel4.enabled);
    }

    #[test]
    fn test_sample_generation() {
        let mut apu = APU::new();

        // Power on and configure for audible output
        apu.write_register(0xFF26, 0x80); // NR52: power on
        apu.write_register(0xFF24, 0x77); // NR50: max volume both sides
        apu.write_register(0xFF25, 0xFF); // NR51: all channels to both outputs

        // Enable channel 1
        apu.write_register(0xFF11, 0x80); // NR11: 50% duty
        apu.write_register(0xFF12, 0xF0); // NR12: volume 15
        apu.write_register(0xFF13, 0xD6); // NR13: freq low (for ~440Hz)
        apu.write_register(0xFF14, 0x87); // NR14: trigger + freq high

        // Step enough to generate some samples
        // At 44100 Hz sample rate and 4194304 Hz CPU clock,
        // we need ~95 T-cycles per sample, so stepping ~1000 M-cycles
        // should give us plenty of samples
        for _ in 0..100 {
            apu.step(10);
        }

        let samples = apu.take_samples();
        assert!(!samples.is_empty(), "APU should have generated samples");

        // Verify samples are in valid range [-1.0, 1.0]
        for (left, right) in &samples {
            assert!(
                *left >= -1.0 && *left <= 1.0,
                "Left sample out of range: {}",
                left
            );
            assert!(
                *right >= -1.0 && *right <= 1.0,
                "Right sample out of range: {}",
                right
            );
        }

        // At least some samples should be non-zero (channel 1 is active)
        let has_nonzero = samples.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "Expected some non-zero audio samples");
    }
}

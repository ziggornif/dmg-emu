#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("[WARN] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        println!("[ERROR] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! print_cpu_state {
    ($cpu:expr) => {
        println!("CPU State:");
        println!(
            "  A: 0x{:02X}  F: 0x{:02X}  AF: 0x{:04X}",
            $cpu.a,
            $cpu.f,
            $cpu.af()
        );
        println!(
            "  B: 0x{:02X}  C: 0x{:02X}  BC: 0x{:04X}",
            $cpu.b,
            $cpu.c,
            $cpu.bc()
        );
        println!(
            "  D: 0x{:02X}  E: 0x{:02X}  DE: 0x{:04X}",
            $cpu.d,
            $cpu.e,
            $cpu.de()
        );
        println!(
            "  H: 0x{:02X}  L: 0x{:02X}  HL: 0x{:04X}",
            $cpu.h,
            $cpu.l,
            $cpu.hl()
        );
        println!("  SP: 0x{:04X}  PC: 0x{:04X}", $cpu.sp, $cpu.pc);
        println!(
            "  Flags: Z:{} N:{} H:{} C:{}",
            if $cpu.flag_z() { "1" } else { "0" },
            if $cpu.flag_n() { "1" } else { "0" },
            if $cpu.flag_h() { "1" } else { "0" },
            if $cpu.flag_c() { "1" } else { "0" }
        );
        println!(
            "  IME: {}",
            if $cpu.interrupts_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        );
    };
}

#[macro_export]
macro_rules! print_ppu_state {
    ($ppu:expr) => {
        println!("PPU State:");
        println!("  LCDC: 0x{:02X}  STAT: 0x{:02X}", $ppu.lcdc, $ppu.stat);
        println!("  LY: {}  LYC: {}", $ppu.ly, $ppu.lyc);
        println!("  SCX: {}  SCY: {}", $ppu.scx, $ppu.scy);
        println!(
            "  Mode: {:?}  Cycles: {}",
            $ppu.get_mode(),
            $ppu.get_cycles()
        );
        println!("  LCD Enabled: {}", $ppu.is_lcd_enabled());
    };
}

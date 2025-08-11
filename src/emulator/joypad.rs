#[derive(Debug, Clone)]
pub struct Joypad {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    pub select_buttons: bool,
    pub select_directions: bool,
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
            select_buttons: false,
            select_directions: false,
        }
    }

    pub fn read_register(&self) -> u8 {
        let mut result = 0xFF;

        if !self.select_buttons {
            result &= !0x20;
        }

        if !self.select_directions {
            result &= !0x10;
        }

        if self.select_buttons {
            if self.a {
                result &= !0x01;
            }
            if self.b {
                result &= !0x02;
            }
            if self.select {
                result &= !0x04;
            }
            if self.start {
                result &= !0x08;
            }
        }

        if self.select_directions {
            if self.right {
                result &= !0x01;
            }
            if self.left {
                result &= !0x02;
            }
            if self.up {
                result &= !0x04;
            }
            if self.down {
                result &= !0x08;
            }
        }

        result
    }

    pub fn write_register(&mut self, value: u8) {
        self.select_buttons = (value & 0x20) == 0;
        self.select_directions = (value & 0x10) == 0;
    }

    pub fn set_button(&mut self, button: JoypadButton, pressed: bool) {
        match button {
            JoypadButton::A => self.a = pressed,
            JoypadButton::B => self.b = pressed,
            JoypadButton::Start => self.start = pressed,
            JoypadButton::Select => self.select = pressed,
            JoypadButton::Up => self.up = pressed,
            JoypadButton::Down => self.down = pressed,
            JoypadButton::Left => self.left = pressed,
            JoypadButton::Right => self.right = pressed,
        }
    }

    pub fn button_pressed(&self) -> bool {
        let buttons_active = self.select_buttons && (self.a || self.b || self.select || self.start);
        let directions_active =
            self.select_directions && (self.right || self.left || self.up || self.down);

        buttons_active || directions_active
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadButton {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

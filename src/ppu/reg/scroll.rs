pub struct ScrollRegister {
    x: u8,
    y: u8,
}

impl ScrollRegister {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn update(&mut self, value: u8, latch: bool) {
        if latch {
            self.y = value;
        } else {
            self.x = value;
        }
    }
}

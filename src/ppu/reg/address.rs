pub struct AddressRegister {
    value: u16,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister { value: 0 }
    }

    pub fn get_addr(&self) -> u16 {
        self.value
    }
    pub fn update(&mut self, value: u8, latch: bool) {
        // High byte first
        if latch {
            self.value |= 0b1111_0000;
            self.value += value as u16;
        } else {
            self.value |= 0b0000_1111;
            self.value += (value as u16) << 8;
        }
    }

    pub fn increment(&mut self, inc: u8) {
        self.value = self.value.wrapping_add(inc as u16);

        if self.get_addr() > 0x3FFF {
            // Mirror down the address
            self.value &= 0b01111111_11111111;
        }
    }
}

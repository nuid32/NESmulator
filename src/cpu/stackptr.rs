// Since stack begins at 0x0100
const STACK: u16 = 0x0100;

// Some kind of init ritual
// https://www.nesdev.org/wiki/CPU_power_up_state#cite_note-reset-stack-push-3
const STACK_RESET: u8 = 0xFD;

#[derive(PartialEq)]
pub struct StackPtr {
    addr: u8,
}

impl StackPtr {
    pub fn new() -> Self {
        Self { addr: STACK_RESET }
    }
    pub fn from_addr(addr: u8) -> Self {
        Self { addr }
    }

    pub fn reset(&mut self) {
        self.addr = STACK_RESET;
    }

    pub fn addr(&self) -> u16 {
        STACK + self.addr as u16
    }
    pub fn rel_addr(&self) -> u8 {
        self.addr
    }

    pub fn set(&mut self, new_addr: u8) {
        self.addr = new_addr;
    }

    // Stack grows down in memory
    pub fn inc(&mut self) {
        self.addr = self.addr.wrapping_sub(1);
    }
    pub fn dec(&mut self) {
        self.addr = self.addr.wrapping_add(1);
    }
}

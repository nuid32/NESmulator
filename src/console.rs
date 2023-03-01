use std::{cell::RefCell, rc::Rc};

use crate::{bus::Bus, cpu::Cpu, rom::Rom};

pub struct Console {
    cpu: Cpu,
    bus: Rc<RefCell<Bus>>,
    cycle: u16,
    scanline: u16,
    frame_complete: bool,
    clock_counter: u8,
}

impl Console {
    pub fn new(rom: Rom) -> Self {
        let bus = Rc::new(RefCell::new(Bus::new(rom)));
        Self {
            cpu: Cpu::new(bus.clone()),
            bus,
            cycle: 0,
            scanline: 0,
            frame_complete: false,
            clock_counter: 0,
        }
    }
}

mod reg;

use crate::rom::Mirroring;
use reg::{AddressRegister, ControlRegister, MaskRegister, ScrollRegister};

use bitflags::bitflags;

/* VSO. ....
   |||| ||||
   |||+-++++- PPU open bus
   ||+------- Sprite overflow
   |+-------- Sprite 0 Hit
   +--------- Vertical blank has started
*/

bitflags! {
  pub struct PpuFlags: u8 {
    const OVERFLOW        = 0b0010_0000;
    const SPRITE_ZERO_HIT = 0b0100_0000;
    const VBLANK_STARTED  = 0b1000_0000;
  }
}

pub struct Ppu {
    address_latch: bool,
    reg_address: AddressRegister,
    reg_control: ControlRegister,
    reg_mask: MaskRegister,
    reg_scroll: ScrollRegister,
    status: PpuFlags,
    chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    vram: [u8; 2048],
    oam_address: u8,
    oam_data: [u8; 256],
    internal_data_buffer: u8,
    mirroring: Mirroring,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Ppu {
            chr_rom,
            address_latch: false,
            reg_address: AddressRegister::new(),
            reg_control: ControlRegister::new(),
            reg_mask: MaskRegister::new(),
            reg_scroll: ScrollRegister::new(),
            status: PpuFlags::from_bits_truncate(0b1010_0000),
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_address: 0,
            oam_data: [0; 64 * 4],
            internal_data_buffer: 0,
            mirroring,
        }
    }

    pub fn write_to_address(&mut self, value: u8) {
        self.reg_address.update(value, self.address_latch);
        self.address_latch = true;
    }

    pub fn write_to_control(&mut self, value: u8) {
        self.reg_control.update(value);
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.reg_mask.update(value);
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.reg_scroll.update(value, self.address_latch);
        self.address_latch = true;
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_address = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }
    pub fn read_from_oam_data(&mut self, addr: u16) -> u8 {
        self.oam_data[addr as usize]
    }

    pub fn write_oam_dma(&mut self, page: &[u8]) {
        // Page size is 256 bytes
        for value in page.iter() {
            self.oam_data[self.oam_address as usize] = *value;
            self.oam_address = self.oam_address.wrapping_add(1);
        }
    }

    pub fn get_status(&mut self) -> u8 {
        self.address_latch = false;
        self.status.bits
    }

    fn increment_vram_addr(&mut self) {
        self.reg_address
            .increment(self.reg_control.vram_addr_increment());
    }
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b1011_1111_1111_1111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x0800,
            (Mirroring::Horizontal, 1) => vram_index - 0x0400,
            (Mirroring::Horizontal, 2) => vram_index - 0x0400,
            (Mirroring::Horizontal, 3) => vram_index - 0x0800,
            _ => vram_index,
        }
    }

    pub fn read(&mut self) -> u8 {
        let addr = self.reg_address.get_addr();
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2007 => unreachable!(),
            0x2008..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => panic!("Address space 0x3000..0x3EFF is not expected to be used"),
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
            _ => panic!("Unexpected access to mirrored space ({:x})", addr),
        }
    }
    pub fn write(&mut self, value: u8) {
        let addr = self.reg_address.get_addr();

        match addr {
            0x0000..=0x1FFF => panic!("Attempt to write to CHR ROM space ({:x})", addr),
            0x2000..=0x2007 => unreachable!(),
            0x2008..=0x2FFF => self.vram[self.mirror_vram_addr(addr) as usize] = value,
            0x3000..=0x3EFF => panic!("Address space 0x3000..0x3EFF is not expected to be used"),
            // These addresses are mirrors
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let addr = addr - 0x10;
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize] = value,
            _ => panic!("Unexpected access to mirrored space ({:x})", addr),
        }
        self.increment_vram_addr();
    }
}

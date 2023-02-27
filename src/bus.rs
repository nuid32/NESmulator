use crate::{ppu::Ppu, rom::Rom};

pub struct Bus {
    cpu_wram: [u8; 2048],
    open_bus: u8,
    prg_rom: Vec<u8>,
    ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_wram: [0; 2048],
            open_bus: 0,
            prg_rom: rom.prg_rom,
            ppu: Ppu::new(rom.chr_rom, rom.screen_mirroring),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }

        self.prg_rom[addr as usize]
    }

    // https://www.youtube.com/watch?v=fWqBmmPQP40&t=41m44s
    // TODO: remove debug code
    pub fn write_initial_pc_addr(&mut self, addr: u16) {
        self.mem_write_u16(0xFFFC, addr);
    }
    pub fn read_initial_pc_addr(&mut self) -> u16 {
        self.mem_read_u16(0xFFFC)
    }

    pub fn mem_read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.mem_read(addr);
        let hi = self.mem_read(addr + 1);

        u16::from_le_bytes([lo, hi])
    }
    pub fn mem_write_u16(&mut self, addr: u16, value: u16) {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;

        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM to it's mirrors end
            0x0000..=0x1FFF => {
                let mirrored_down_addr = addr & 0b0000_0111_1111_1111;
                let value = self.cpu_wram[mirrored_down_addr as usize];
                self.open_bus = value;
                value
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address ({:x})", addr);
            }
            // Status
            0x2002 => self.ppu.get_status(),
            // OAM data
            0x2004 => self.ppu.read_from_oam_data(addr),
            // Data
            0x2007 => self.ppu.read(),
            // Mirrors of PPU's registers
            0x2008..=0x3FFF => {
                let mirrored_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_read(mirrored_down_addr)
            }
            // ROM PRG
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => self.open_bus,
        }
    }

    pub fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM to it's mirrors end
            0x0000..=0x1FFF => {
                // XXX Probably mistake here with this AND operation
                let mirrored_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_wram[mirrored_down_addr as usize] = value;
            }
            // Control
            0x2000 => self.ppu.write_to_control(value),
            // Mask
            0x2001 => self.ppu.write_to_mask(value),
            // OAM data
            0x2004 => self.ppu.write_to_oam_data(value),
            // Scroll
            0x2005 => self.ppu.write_to_scroll(value),
            // Address
            0x2006 => self.ppu.write_to_address(value),
            // Data
            0x2007 => self.ppu.write(value),
            // Mirrors of PPU's registers
            0x2008..=0x3FFF => {
                let mirrored_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_write(mirrored_down_addr, value);
            }
            // OAM DMA
            0x4014 => {
                // Copy all from XX00 to XXFF to the PPU's OAM
                let page = &self.cpu_wram[((value as usize) << 8)..((value as usize) << 8 + 256)];
                self.ppu.write_oam_dma(page);
            }
            // ROM PRG
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space ({:x})", addr)
            }
            _ => {
                panic!("Attempt to write to {:x}", addr);
            }
        }
    }
}

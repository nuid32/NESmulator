use crate::{ppu::Ppu, rom::Rom};

pub struct Bus {
    cpu_wram: [u8; 2048],
    open_bus: u8,
    ppu_open_bus: u8,
    prg_rom: Vec<u8>,
    ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_wram: [0; 2048],
            open_bus: 0,
            ppu_open_bus: 0,
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

    // XXX Maybe I misunderstood open bus behavior
    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM to it's mirrors end
            0x0000..=0x1FFF => {
                let mirrored_down_addr = addr & 0b0000_0111_1111_1111;
                let value = self.cpu_wram[mirrored_down_addr as usize];
                self.open_bus = value;
                value
            }
            // Write-only PPU ports
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => {
                self.open_bus = self.ppu_open_bus;
                self.ppu_open_bus
            }
            // Status
            0x2002 => {
                let status = self.ppu.get_status();

                // Reading the PPU's status port loads bits 7-5 only
                self.ppu_open_bus &= 0b0001_1111;
                self.ppu_open_bus |= status;
                self.open_bus = status;

                status
            }
            // OAM data
            0x2004 => {
                let value = self.ppu.read_from_oam_data(addr);
                self.ppu_open_bus = value;
                self.open_bus = value;
                value
            }
            // Data
            0x2007 => {
                let value = self.ppu.read();
                self.ppu_open_bus = value;
                self.open_bus = value;
                value
            }
            // Mirrors of PPU's registers
            0x2008..=0x3FFF => {
                let mirrored_down_addr = addr & 0b0010_0000_0000_0111;
                // Open bus will be modified after mirrored read
                self.mem_read(mirrored_down_addr)
            }
            0x4016..=0x4017 => {
                todo!("Emulate joypads and open bus behavior (affects bits 4-0 only) here")
            }
            // ROM PRG
            0x8000..=0xFFFF => {
                let value = self.read_prg_rom(addr);
                self.open_bus = value;
                value
            }
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
            0x2000 => {
                self.ppu_open_bus = value;
                self.ppu.write_to_control(value);
            }
            // Mask
            0x2001 => {
                self.ppu_open_bus = value;
                self.ppu.write_to_mask(value);
            }
            // Status (read-only)
            0x2002 => self.ppu_open_bus = value,
            // OAM data
            0x2004 => {
                self.ppu_open_bus = value;
                self.ppu.write_to_oam_data(value);
            }
            // Scroll
            0x2005 => {
                self.ppu_open_bus = value;
                self.ppu.write_to_scroll(value);
            }
            // Address
            0x2006 => {
                self.ppu_open_bus = value;
                self.ppu.write_to_address(value);
            }
            // Data
            0x2007 => {
                self.ppu_open_bus = value;
                self.ppu.write(value);
            }
            // Mirrors of PPU's registers
            0x2008..=0x3FFF => {
                let mirrored_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_write(mirrored_down_addr, value);
            }
            // OAM DMA
            0x4014 => {
                self.ppu_open_bus = value;
                // Copy all from XX00 to XXFF to the PPU's OAM
                let page = &self.cpu_wram[((value as usize) << 8)..((value as usize) << 8 + 256)];
                self.ppu.write_oam_dma(page);
            }
            // ROM PRG
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space ({:x})", addr)
            }
            _ => {
                panic!("Attempt to write to {:x}", addr)
            }
        }
    }
}

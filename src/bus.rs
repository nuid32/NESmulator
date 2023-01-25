use crate::{memory::Memory, rom::Rom};

// About mirroring: https://www.nesdev.org/wiki/Mirroring
const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;

const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const ROM_PRG_BEGINNING: u16 = 0x8000;
const ROM_PRG_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_vram: [u8; 2048],
    rom: Option<Rom>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom: None,
        }
    }

    pub fn insert_rom(&mut self, rom: Rom) {
        self.rom = Some(rom);
    }

    pub fn remove_rom(&mut self) {
        self.rom = None;
    }

    // Set 0xFFFC for PC

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        let prg_rom: &Vec<u8> = self
            .rom
            .as_ref()
            .expect("There is no ROM inserted")
            .prg_rom
            .as_ref();

        addr -= 0x8000;
        if prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }

        prg_rom[addr as usize]
    }

    // https://www.youtube.com/watch?v=fWqBmmPQP40&t=41m44s
    pub fn write_init_pc_addr(&mut self, addr: u16) {
        self.mem_write_u16(0xFFFC, addr);
    }
    pub fn read_init_pc_addr(&mut self) -> u16 {
        self.mem_read_u16(0xFFFC)
    }
}

impl Memory for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            // TODO Implement after implementing PPU emulation
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                // let mirror_down_addr = addr & 0b00100000_00000111;
                0
            }
            ROM_PRG_BEGINNING..=ROM_PRG_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring memory read access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = value;
            }
            // TODO Implement after implementing PPU emulation
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                // let mirror_down_addr = addr & 0b0010_0000_0000_0111;
            }
            ROM_PRG_BEGINNING..=ROM_PRG_END => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring memory write access at {}", addr);
            }
        }
    }
}

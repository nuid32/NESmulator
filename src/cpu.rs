use crate::opcode::OPCODES_MAP;
use bitflags::bitflags;

bitflags! {
// NV_B DIZC
// || | ||||
// || | |||+- Carry
// || | ||+-- Zero
// || | |+--- Interrupt Disable
// || | +---- Decimal
// || +------ Break
// |+-------- Overflow
// +--------- Negative
    pub struct CpuFlags: u8 {
        const CARRY             = 0b0000_0001;
        const ZERO              = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL_MODE      = 0b0000_1000;
        const BREAK             = 0b0001_0000;
        const BREAK2            = 0b0010_0000;
        const OVERFLOW          = 0b0100_0000;
        const NEGATIVE          = 0b1000_0000;
    }
}

// Since stack begins at 0x0100
const STACK: u16 = 0x0100;
// Some kind of init ritual
// https://www.nesdev.org/wiki/CPU_power_up_state#cite_note-reset-stack-push-3
const STACK_RESET: u8 = 0xFD;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack_pointer: u8,
    pub status: CpuFlags,
    pub program_counter: u16,
    memory: [u8; 0xFFFF], // 65536 cells
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: STACK_RESET,
            status: CpuFlags::from_bits_truncate(0b00100100),
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos + 1);

        u16::from_le_bytes([lo, hi])
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;

        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                let deref_base = u16::from_le_bytes([lo, hi]);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_RESET;
        self.status = CpuFlags::from_bits_truncate(0b0010_0100);

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let instruction = OPCODES_MAP
                .get(&opcode)
                .expect(&format!("Opcode {:x} is not recognized", opcode));

            let program_counter_old = self.program_counter;

            match opcode {
                // BRK
                0x00 => {
                    return;
                }

                // INX
                0xE8 => self.inx(),

                // TAX
                0xAA => self.tax(),
                // TAY
                0xA8 => self.tay(),
                // TSX
                0xBA => self.tsx(),
                // TXA
                0x8A => self.txa(),
                // TXS
                0x9A => self.txs(),
                // TYA
                0x98 => self.tya(),

                // CLC
                0x18 => self.clear_carry_flag(),
                // CLD
                0xD8 => self.status.remove(CpuFlags::DECIMAL_MODE),
                // CLI
                0x58 => self.status.remove(CpuFlags::INTERRUPT_DISABLE),
                // CLV
                0xB8 => self.status.remove(CpuFlags::OVERFLOW),

                // SEC
                0x38 => self.set_carry_flag(),
                // SED
                0xF8 => self.status.insert(CpuFlags::DECIMAL_MODE),
                // SEI
                0x78 => self.status.insert(CpuFlags::INTERRUPT_DISABLE),

                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&instruction.addressing_mode)
                }
                // LDX
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(&instruction.addressing_mode),

                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&instruction.addressing_mode)
                }
                // STX
                0x86 | 0x96 | 0x8E => self.stx(&instruction.addressing_mode),
                // STY
                0x84 | 0x94 | 0x8C => self.sty(&instruction.addressing_mode),

                // ASL
                0x0A | 0x06 | 0x16 | 0x0E | 0x1E => self.asl(&instruction.addressing_mode),
                // LSR
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => self.lsr(&instruction.addressing_mode),
                _ => unimplemented!(),
            }

            if self.program_counter == program_counter_old {
                self.program_counter += instruction.bytes as u16 - 1;
            }
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    fn set_carry_flag(&mut self) {
        self.status.insert(CpuFlags::CARRY);
    }
    fn clear_carry_flag(&mut self) {
        self.status.remove(CpuFlags::CARRY);
    }

    // Increment X Register
    fn inx(&mut self) {
        // With overflow emulation
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Transfer Accumulator to X
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    // Transfer Accumulator to Y
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }
    // Transfer Stack Pointer to X
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }
    // Transfer X to Accumulator
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }
    // Transfer X to Stack Pointer
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }
    // Transfer Y to Accumulator
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Load Accumulator
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    // Load X Register
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Store Accumulator
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }
    // Store X Register
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }
    // Store Y Register
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    // Arithmetic Shift Left
    fn asl(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::NoneAddressing => {
                self.register_a = self.register_a << 1;
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                value = value << 1;

                self.mem_write(addr, value);
                self.update_zero_and_negative_flags(value);
            }
        }
    }
    // Logical Shift Right
    fn lsr(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::NoneAddressing => {
                self.register_a = self.register_a >> 1;
                self.update_zero_and_negative_flags(self.register_x);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                if value & 1 == 1 {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }
                value = value >> 1;

                self.mem_write(addr, value);
                self.update_zero_and_negative_flags(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0x00]);

        assert!(cpu.status.contains(CpuFlags::ZERO));
    }
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x08, 0x01);

        cpu.load_and_run(vec![0xA5, 0x08, 0x00]);

        assert_eq!(cpu.register_a, 0x01);
    }

    #[test]
    fn test_ldx_zero_flag() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xA2, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }
    #[test]
    fn test_ldx_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x08, 0x18);

        cpu.load_and_run(vec![0xA6, 0x08, 0x00]);
        assert!(cpu.register_x == 0x18);
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAA, 0x00]);
        cpu.reset();
        cpu.register_a = 8;
        cpu.run();

        assert!(cpu.register_x == 8);
    }
    #[test]
    fn test_tay() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xA8, 0x00]);
        cpu.reset();
        cpu.register_a = 8;
        cpu.run();

        assert!(cpu.register_y == 8);
    }
    #[test]
    fn test_tsx() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xBA, 0x00]);
        cpu.reset();
        cpu.stack_pointer = 8;
        cpu.run();

        assert!(cpu.register_x == 8);
    }
    #[test]
    fn test_txa() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x8A, 0x00]);
        cpu.reset();
        cpu.register_x = 8;
        cpu.run();

        assert!(cpu.register_a == 8);
    }
    #[test]
    fn test_txs() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x9A, 0x00]);
        cpu.reset();
        cpu.register_x = 8;
        cpu.run();

        assert!(cpu.stack_pointer == 8);
    }
    #[test]
    fn test_tya() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x98, 0x00]);
        cpu.reset();
        cpu.register_y = 8;
        cpu.run();

        assert!(cpu.register_a == 8);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE8, 0xE8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xFF;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x95, 0x08, 0x00]);
        cpu.reset();
        cpu.register_a = 17;
        cpu.run();

        assert_eq!(cpu.memory[0x08], 17);
    }
    #[test]
    fn test_stx() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x96, 0x08, 0x00]);
        cpu.reset();
        cpu.register_x = 17;
        cpu.run();

        assert_eq!(cpu.memory[0x08], 17);
    }
    #[test]
    fn test_sty() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x94, 0x08, 0x00]);
        cpu.reset();
        cpu.register_y = 17;
        cpu.run();

        assert_eq!(cpu.memory[0x08], 17);
    }

    #[test]
    fn test_sec() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0x00]);

        assert!(cpu.status.contains(CpuFlags::CARRY));
    }
    #[test]
    fn test_sed() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xF8, 0x00]);

        assert!(cpu.status.contains(CpuFlags::DECIMAL_MODE));
    }
    #[test]
    fn test_sei() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x78, 0x00]);

        assert!(cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x18, 0x00]);
        cpu.reset();
        cpu.set_carry_flag();
        cpu.run();

        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }
    #[test]
    fn test_cld() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xD8, 0x00]);
        cpu.reset();
        cpu.status.insert(CpuFlags::DECIMAL_MODE);
        cpu.run();

        assert!(!cpu.status.contains(CpuFlags::DECIMAL_MODE));
    }
    #[test]
    fn test_cli() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x58, 0x00]);
        cpu.reset();
        cpu.status.insert(CpuFlags::INTERRUPT_DISABLE);
        cpu.run();

        assert!(!cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
    }
    #[test]
    fn test_clv() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xB8, 0x00]);
        cpu.reset();
        cpu.status.insert(CpuFlags::OVERFLOW);
        cpu.run();

        assert!(!cpu.status.contains(CpuFlags::OVERFLOW));
    }

    #[test]
    fn test_asl_reg_a() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x0A, 0x00]);
        cpu.reset();
        cpu.register_a = 0b1000_0000;
        cpu.run();

        assert_eq!(cpu.register_a, 0);
    }
    #[test]
    fn test_asl() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x06, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x10, 0b1000_0000);
        cpu.run();
        let value = cpu.mem_read(0x10);

        assert_eq!(value, 0);
    }
}

use super::{opcode::OPCODES_MAP, stackptr::StackPtr};
use bitflags::bitflags;

#[cfg(test)]
#[path = "cpu_tests.rs"]
mod cpu_tests;

bitflags! {
// NV2B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ Break
// ||+------- Break2
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
    pub stackptr: StackPtr,
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
            stackptr: StackPtr::new(),
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

    fn branch(&mut self) {
        let jump = self.mem_read(self.program_counter) as i8;
        let jump_addr = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(jump as u16);

        self.program_counter = jump_addr;
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

    fn stack_push(&mut self, value: u8) {
        self.mem_write(self.stackptr.addr(), value);
        self.stackptr.inc();
    }
    fn stack_push_u16(&mut self, value: u16) {
        self.mem_write_u16(self.stackptr.addr(), value);
        self.stackptr.inc();
    }
    fn stack_pop(&mut self) -> u8 {
        self.stackptr.dec();
        self.mem_read(self.stackptr.addr())
    }
    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        hi << 8 | lo
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn set_register_x(&mut self, value: u8) {
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn set_register_y(&mut self, value: u8) {
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
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
        self.stackptr.reset();
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
                // ROL
                0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.rol(&instruction.addressing_mode),
                // ROR
                0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ror(&instruction.addressing_mode),

                // PHA
                0x48 => self.pha(&instruction.addressing_mode),
                // PLA
                0x68 => self.pla(&instruction.addressing_mode),

                // PHP
                0x08 => self.php(&instruction.addressing_mode),
                // PLP
                0x28 => self.plp(&instruction.addressing_mode),

                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&instruction.addressing_mode)
                }
                // ORA
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&instruction.addressing_mode)
                }
                // EOR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&instruction.addressing_mode)
                }

                // BPL
                0x10 => self.bpl(&instruction.addressing_mode),
                // BMI
                0x30 => self.bmi(&instruction.addressing_mode),
                // BVC
                0x50 => self.bvc(&instruction.addressing_mode),
                // BVS
                0x70 => self.bvs(&instruction.addressing_mode),
                // BCC
                0x90 => self.bcc(&instruction.addressing_mode),
                // BCS
                0xB0 => self.bcs(&instruction.addressing_mode),
                // BNE
                0xD0 => self.bne(&instruction.addressing_mode),
                // BEQ
                0xF0 => self.beq(&instruction.addressing_mode),

                _ => unimplemented!(),
            }

            if self.program_counter == program_counter_old {
                self.program_counter += instruction.bytes as u16 - 1;
            }
        }
    }

    // Increment X Register
    fn inx(&mut self) {
        // With overflow emulation
        self.set_register_x(self.register_x.wrapping_add(1));
    }

    // Transfer Accumulator to X
    fn tax(&mut self) {
        self.set_register_x(self.register_a);
    }
    // Transfer Accumulator to Y
    fn tay(&mut self) {
        self.set_register_y(self.register_a);
    }
    // Transfer Stack Pointer to X
    fn tsx(&mut self) {
        self.set_register_x(self.stackptr.rel_addr());
    }
    // Transfer X to Accumulator
    fn txa(&mut self) {
        self.set_register_a(self.register_x);
    }
    // Transfer X to Stack Pointer
    fn txs(&mut self) {
        self.stackptr.set(self.register_x);
    }
    // Transfer Y to Accumulator
    fn tya(&mut self) {
        self.set_register_a(self.register_y);
    }

    // Load Accumulator
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value);
    }
    // Load X Register
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_x(value);
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
                self.set_register_a(self.register_a << 1);
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
                self.set_register_a(self.register_a >> 1);
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
    // Rotate left
    fn rol(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::NoneAddressing => {
                let mut value = self.register_a;
                let old_carry = self.status.contains(CpuFlags::CARRY);

                // Is bit 7 set
                if value & 0b1000_0000 == 0b1000_0000 {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }

                value = value << 1;
                if old_carry {
                    value = value | 1;
                }

                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                let old_carry = self.status.contains(CpuFlags::CARRY);

                // Is bit 7 set
                if value & 0b1000_0000 == 0b1000_0000 {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }

                value = value << 1;
                if old_carry {
                    value = value | 1;
                }

                self.mem_write(addr, value);
                self.update_zero_and_negative_flags(value);
            }
        }
    }
    // Rotate right
    fn ror(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::NoneAddressing => {
                let mut value = self.register_a;
                let old_carry = self.status.contains(CpuFlags::CARRY);

                // Is bit 0 set
                if value & 1 == 1 {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }

                value = value >> 1;
                if old_carry {
                    value = value | 0b1000_0000;
                }
                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                let old_carry = self.status.contains(CpuFlags::CARRY);

                if value & 1 == 1 {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }

                value = value >> 1;
                if old_carry {
                    value = value | 0b1000_0000;
                }

                self.mem_write(addr, value);
                self.update_zero_and_negative_flags(value);
            }
        }
    }

    // Push Accumulator
    fn pha(&mut self, mode: &AddressingMode) {
        self.stack_push(self.register_a);
    }
    // Pull Accumulator
    fn pla(&mut self, mode: &AddressingMode) {
        let value = self.stack_pop();
        self.set_register_a(value);
    }

    // Push Processor Status
    fn php(&mut self, mode: &AddressingMode) {
        let mut flags = self.status.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits);
    }
    // Pull Processor Status
    fn plp(&mut self, mode: &AddressingMode) {
        self.status.bits = self.stack_pop();
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);
    }

    // Logical AND
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(self.register_a & value);
    }
    // Logical Inclusive OR
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(value | self.register_a);
    }
    // Exclusive OR
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(value ^ self.register_a);
    }

    // Branch if Positive
    fn bpl(&mut self, mode: &AddressingMode) {
        if !self.status.contains(CpuFlags::NEGATIVE) {
            self.branch();
        }
    }
    // Branch if Minus
    fn bmi(&mut self, mode: &AddressingMode) {
        if self.status.contains(CpuFlags::ZERO) {
            self.branch();
        }
    }
    // Branch if Overflow Clear
    fn bvc(&mut self, mode: &AddressingMode) {
        if !self.status.contains(CpuFlags::OVERFLOW) {
            self.branch();
        }
    }
    // Branch if Overflow Set
    fn bvs(&mut self, mode: &AddressingMode) {
        if self.status.contains(CpuFlags::OVERFLOW) {
            self.branch();
        }
    }
    // Branch if Carry Clear
    fn bcc(&mut self, mode: &AddressingMode) {
        if !self.status.contains(CpuFlags::CARRY) {
            self.branch();
        }
    }
    // Branch if Carry Set
    fn bcs(&mut self, mode: &AddressingMode) {
        if self.status.contains(CpuFlags::CARRY) {
            self.branch();
        }
    }
    // Branch if Not Equal
    fn bne(&mut self, mode: &AddressingMode) {
        if !self.status.contains(CpuFlags::ZERO) {
            self.branch();
        }
    }
    // Branch if Equal
    fn beq(&mut self, mode: &AddressingMode) {
        if self.status.contains(CpuFlags::ZERO) {
            self.branch();
        }
    }
}

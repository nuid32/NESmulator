use crate::{bus::Bus, memory::Memory};

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
    pub struct CpuFlag: u8 {
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
    pub status: CpuFlag,
    pub pc: u16, // Program Counter
    pub bus: Bus,
}

impl Memory for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }
    fn mem_read_u16(&self, addr: u16) -> u16 {
        self.bus.mem_read_u16(addr)
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        self.bus.mem_write(addr, value);
    }
    fn mem_write_u16(&mut self, addr: u16, value: u16) {
        self.bus.mem_write_u16(addr, value);
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stackptr: StackPtr::new(),
            status: CpuFlag::from_bits_truncate(0b00100100),
            pc: 0,
            bus: Bus::new(),
        }
    }

    fn branch(&mut self) {
        let jump = self.mem_read(self.pc) as i8;
        let jump_addr = self.pc.wrapping_add(1).wrapping_add(jump as u16);

        self.pc = jump_addr;
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.set(CpuFlag::ZERO, result == 0);

        self.status
            .set(CpuFlag::NEGATIVE, result & 0b1000_0000 != 0);
    }

    fn set_flag(&mut self, flag: CpuFlag) {
        self.status.insert(flag);
    }
    fn clear_flag(&mut self, flag: CpuFlag) {
        self.status.remove(flag);
    }

    fn stack_push(&mut self, value: u8) {
        self.mem_write(self.stackptr.addr(), value);
        self.stackptr.inc();
    }
    fn stack_push_u16(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
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
            AddressingMode::Immediate => self.pc,
            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.pc),

            AddressingMode::ZeroPage_X => {
                let addr = self.mem_read(self.pc);
                let addr = addr.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_read(self.pc);
                let addr = addr.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.pc);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.pc);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.pc);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.pc);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
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
        self.status = CpuFlag::from_bits_truncate(0b0010_0100);

        self.pc = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0600 + i, program[i as usize]);
        }
        self.mem_write_u16(0xFFFC, 0x0600);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            let opcode = self.mem_read(self.pc);
            self.pc += 1;
            let program_counter_old = self.pc;

            let instr = OPCODES_MAP
                .get(&opcode)
                .expect(&format!("Opcode {:x} is not recognized", opcode));

            match opcode {
                // BRK
                0x00 => return,
                //NOP
                0xEA => (),
                // BIT
                0x24 | 0x2C => self.bit(&instr.addressing_mode),

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
                0x18 => self.clear_flag(CpuFlag::CARRY),
                // CLD
                0xD8 => self.clear_flag(CpuFlag::DECIMAL_MODE),
                // CLI
                0x58 => self.clear_flag(CpuFlag::INTERRUPT_DISABLE),
                // CLV
                0xB8 => self.clear_flag(CpuFlag::OVERFLOW),

                // SEC
                0x38 => self.set_flag(CpuFlag::CARRY),
                // SED
                0xF8 => self.set_flag(CpuFlag::DECIMAL_MODE),
                // SEI
                0x78 => self.set_flag(CpuFlag::INTERRUPT_DISABLE),

                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&instr.addressing_mode)
                }
                // LDX
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(&instr.addressing_mode),
                // LDY
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(&instr.addressing_mode),

                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(&instr.addressing_mode),
                // STX
                0x86 | 0x96 | 0x8E => self.stx(&instr.addressing_mode),
                // STY
                0x84 | 0x94 | 0x8C => self.sty(&instr.addressing_mode),

                // ASL
                0x0A | 0x06 | 0x16 | 0x0E | 0x1E => self.asl(&instr.addressing_mode),
                // LSR
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => self.lsr(&instr.addressing_mode),
                // ROL
                0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.rol(&instr.addressing_mode),
                // ROR
                0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ror(&instr.addressing_mode),

                // PHA
                0x48 => self.pha(),
                // PLA
                0x68 => self.pla(),

                // PHP
                0x08 => self.php(),
                // PLP
                0x28 => self.plp(),

                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&instr.addressing_mode)
                }
                // ORA
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&instr.addressing_mode)
                }
                // EOR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&instr.addressing_mode)
                }

                // BPL
                0x10 => self.bpl(),
                // BMI
                0x30 => self.bmi(),
                // BVC
                0x50 => self.bvc(),
                // BVS
                0x70 => self.bvs(),
                // BCC
                0x90 => self.bcc(),
                // BCS
                0xB0 => self.bcs(),
                // BNE
                0xD0 => self.bne(),
                // BEQ
                0xF0 => self.beq(),

                // CMP
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&instr.addressing_mode)
                }
                // CPX
                0xE0 | 0xE4 | 0xEC => self.cpx(&instr.addressing_mode),
                // CPY
                0xC0 | 0xC4 | 0xCC => self.cpy(&instr.addressing_mode),

                // INC
                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&instr.addressing_mode),
                // INX
                0xE8 => self.inx(),
                // INY
                0xC8 => self.iny(),

                // DEC
                0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(&instr.addressing_mode),
                // DEX
                0xCA => self.dex(),
                // DEY
                0x88 => self.dey(),

                // JMP
                0x4C | 0x6C => self.jmp(&instr.addressing_mode),
                // JSR
                0x20 => self.jsr(),
                // RTI
                0x40 => self.rti(),
                // RTS
                0x60 => self.rts(),

                // ADC
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&instr.addressing_mode)
                }
                // SBC
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&instr.addressing_mode)
                }

                _ => unimplemented!(),
            }

            if self.pc == program_counter_old {
                self.pc += (instr.bytes - 1) as u16;
            }

            callback(self)
        }
    }

    // Bit Test
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.status.set(CpuFlag::ZERO, value & self.register_a == 0);
        self.status
            .set(CpuFlag::NEGATIVE, value & 0b1000_0000 == 0b1000_0000);
        self.status
            .set(CpuFlag::OVERFLOW, value & 0b0100_0000 == 0b0100_0000);
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
    // Load Y Register
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_y(value);
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
                let mut value = self.register_a;
                self.status.set(CpuFlag::CARRY, value >> 7 == 1);

                value = value << 1;
                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                self.status.set(CpuFlag::CARRY, value >> 7 == 1);

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
                // self.set_register_a(self.register_a >> 1);
                let mut value = self.register_a;
                self.status.set(CpuFlag::CARRY, value & 1 == 1);

                value = value >> 1;
                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                self.status.set(CpuFlag::CARRY, value & 1 == 1);

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
                let old_carry = self.status.contains(CpuFlag::CARRY);

                // Is bit 7 set
                self.status.set(CpuFlag::CARRY, value >> 7 == 1);

                value = value << 1;
                if old_carry {
                    value = value | 1;
                }

                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                let old_carry = self.status.contains(CpuFlag::CARRY);

                self.status.set(CpuFlag::CARRY, value >> 7 == 1);

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
                let old_carry = self.status.contains(CpuFlag::CARRY);

                // Is bit 0 set
                self.status.set(CpuFlag::CARRY, value & 1 == 1);

                value = value >> 1;
                if old_carry {
                    value = value | 0b1000_0000;
                }
                self.set_register_a(value);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let mut value = self.mem_read(addr);
                let old_carry = self.status.contains(CpuFlag::CARRY);

                self.status.set(CpuFlag::CARRY, value & 1 == 1);

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
    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }
    // Pull Accumulator
    fn pla(&mut self) {
        let value = self.stack_pop();
        self.set_register_a(value);
    }

    // Push Processor Status
    fn php(&mut self) {
        let mut flags = self.status.clone();
        flags.insert(CpuFlag::BREAK);
        flags.insert(CpuFlag::BREAK2);
        self.stack_push(flags.bits());
    }
    // Pull Processor Status
    fn plp(&mut self) {
        self.status.bits = self.stack_pop();
        self.clear_flag(CpuFlag::BREAK);
        self.set_flag(CpuFlag::BREAK2);
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
        self.set_register_a(self.register_a | value);
    }
    // Exclusive OR
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(self.register_a ^ value);
    }

    // Branch if Positive
    fn bpl(&mut self) {
        if !self.status.contains(CpuFlag::NEGATIVE) {
            self.branch();
        }
    }
    // Branch if Minus
    fn bmi(&mut self) {
        if self.status.contains(CpuFlag::NEGATIVE) {
            self.branch();
        }
    }
    // Branch if Overflow Clear
    fn bvc(&mut self) {
        if !self.status.contains(CpuFlag::OVERFLOW) {
            self.branch();
        }
    }
    // Branch if Overflow Set
    fn bvs(&mut self) {
        if self.status.contains(CpuFlag::OVERFLOW) {
            self.branch();
        }
    }
    // Branch if Carry Clear
    fn bcc(&mut self) {
        if !self.status.contains(CpuFlag::CARRY) {
            self.branch();
        }
    }
    // Branch if Carry Set
    fn bcs(&mut self) {
        if self.status.contains(CpuFlag::CARRY) {
            self.branch();
        }
    }
    // Branch if Not Equal
    fn bne(&mut self) {
        if !self.status.contains(CpuFlag::ZERO) {
            self.branch();
        }
    }
    // Branch if Equal
    fn beq(&mut self) {
        if self.status.contains(CpuFlag::ZERO) {
            self.branch();
        }
    }

    // Compare
    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.status.set(CpuFlag::CARRY, self.register_a >= value);

        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(value));
    }
    // Compare X Register
    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.status.set(CpuFlag::CARRY, self.register_x >= value);

        self.update_zero_and_negative_flags(self.register_x.wrapping_sub(value));
    }
    // Compare Y Register
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.status.set(CpuFlag::CARRY, self.register_y >= value);

        self.update_zero_and_negative_flags(self.register_y.wrapping_sub(value));
    }

    // Increment Memory
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_add(1);

        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }
    // Increment X Register
    fn inx(&mut self) {
        self.set_register_x(self.register_x.wrapping_add(1));
    }
    // Increment Y Register
    fn iny(&mut self) {
        self.set_register_y(self.register_y.wrapping_add(1));
    }

    // Decrement Memory
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_sub(1);

        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }
    // Decrement X Register
    fn dex(&mut self) {
        self.set_register_x(self.register_x.wrapping_sub(1));
    }
    // Decrement Y Register
    fn dey(&mut self) {
        self.set_register_y(self.register_y.wrapping_sub(1));
    }

    // Jump
    fn jmp(&mut self, mode: &AddressingMode) {
        match mode {
            AddressingMode::Absolute => {
                let addr = self.mem_read_u16(self.pc);
                self.pc = addr;
            }
            AddressingMode::NoneAddressing => {
                let addr = self.mem_read_u16(self.pc);

                // 6502 does not correctly fetch the target address if indirect vector falls on a page boundary
                // (e.g. $xxFF where xx is any value from $00 to $FF). In this case it fetches the LSB from $xxFF as expected
                // but takes the MSB from $xx00. Fixed in some later chips.
                let indirect_ref = if addr & 0x00FF == 0x00FF {
                    let lo = self.mem_read(addr);
                    let hi = self.mem_read(addr & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.mem_read_u16(addr)
                };

                self.pc = indirect_ref;
            }
            _ => unreachable!(),
        }
    }
    // Jump to Subroutine
    fn jsr(&mut self) {
        self.stack_push_u16(self.pc + 2 - 1);
        let addr = self.mem_read_u16(self.pc);
        self.pc = addr;
    }
    // Return from Interrupt
    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.clear_flag(CpuFlag::BREAK);
        self.set_flag(CpuFlag::BREAK2);

        self.pc = self.stack_pop_u16();
    }
    // Return from Subroutine
    fn rts(&mut self) {
        self.pc = self.stack_pop_u16() + 1;
    }

    fn add_to_register_a(&mut self, value: u8) {
        let mut sum = self.register_a as u16 + value as u16;

        // If overflow occurs the CARRY bit is clear and this enables multiple byte addition/substraction to be performed
        if self.status.contains(CpuFlag::CARRY) {
            sum += 1;
        }
        let carry = sum > 0xff;
        if carry {
            self.status.insert(CpuFlag::CARRY);
        } else {
            self.status.remove(CpuFlag::CARRY);
        }

        let result = sum as u8;

        if (value ^ result) & (result ^ self.register_a) & 0b1000_0000 != 0 {
            self.status.insert(CpuFlag::OVERFLOW);
        } else {
            self.status.remove(CpuFlag::OVERFLOW)
        }

        self.set_register_a(result);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(((value as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(value);
    }
}

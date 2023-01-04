#[cfg(test)]
mod cpu_tests {

    use super::super::*;

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
        cpu.stackptr = StackPtr::from_addr(8);
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

        assert!(cpu.stackptr.rel_addr() == 8);
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

    #[test]
    fn test_rol_reg_a() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x2A, 0x00]);
        cpu.reset();
        cpu.register_a = 0b1100_0000;
        cpu.run();

        // If carry flag wasn't set before, then bit 0 will be 0
        assert_eq!(cpu.register_a, 128);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }
    #[test]
    fn test_rol() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x26, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x10, 0b1100_0000);
        cpu.run();
        let value = cpu.mem_read(0x10);

        assert_eq!(value, 128);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_ror_reg_a() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x6A, 0x00]);
        cpu.reset();
        cpu.register_a = 0b0000_0011;
        cpu.set_carry_flag();
        cpu.run();

        // If carry flag wasn't set before, then bit 0 will be 0
        assert_eq!(cpu.register_a, 129);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }
    #[test]
    fn test_ror() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x66, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x10, 0b0000_0011);
        cpu.set_carry_flag();
        cpu.run();
        let value = cpu.mem_read(0x10);

        assert_eq!(value, 129);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x48, 0x00]);
        cpu.reset();
        cpu.register_a = 8;
        cpu.run();
        let value = cpu.stack_pop();

        assert_eq!(value, 8);
    }
    #[test]
    fn test_pla() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x68, 0x00]);
        cpu.reset();
        cpu.stack_push(8);
        cpu.run();

        assert_eq!(cpu.register_a, 8);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x08, 0x00]);
        cpu.reset();
        cpu.status.bits = 0b1100_0000;
        cpu.run();
        let value = cpu.stack_pop();

        // PHP instruction must set 4 and 5 bits in status, BREAK and BREAK2 respectively
        assert_eq!(value, 0b1111_0000);
    }
    #[test]
    fn test_plp() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x28, 0x00]);
        cpu.reset();
        cpu.stack_push(0b0001_0000);
        cpu.run();

        // PLP sets 4 bit to 0 and 5 bit to 1 while pulling. BREAK is 0, BREAK2 is 1
        assert_eq!(cpu.status.bits, 0b0010_0000);
    }
}

use std::ops::Add;

use crate::opcode::OpCode;

#[derive(Debug, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

const ZERO_FLAG: u8 = 0b0000_0010;
const NEGATIVE_FLAG: u8 = 0b1000_0000;
const CARRY_FLAG: u8 = 0b0000_0001;
const OVERFLOW_FLAG: u8 = 0b0100_0000;

pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xffff],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xffff],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    // following two functions implement little endianness
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        // 0xfffc is where the program counter start address is read from
        self.mem_write_u16(0xfffc, 0x8000);
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xfffc);
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            // immediate: current PC value
            AddressingMode::Immediate => self.program_counter,
            // zeropage: can only access first byte of addresses
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            // absolute: full memory location
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            // zeropagex: first byte of addresses, but adds the value of X to the address first
            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.x) as u16
            }
            // same as above but Y register
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.y) as u16
            }
            // absolute addressing but adding X and Y registers as above
            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.y as u16)
            }
            // indirectx: take a zeropage address, add the value of X, look up the 2 byte address
            // ??? why are you like this
            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);
                let ptr = (base as u8).wrapping_add(self.x);
                self.mem_read_u16(ptr as u16)
            }
            // indirecty: zeropage address is dereferenced, then Y is added to the address
            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);
                let deref_base = self.mem_read_u16(base as u16);
                deref_base.wrapping_add(self.y as u16)
            }
            AddressingMode::NoneAddressing => {
                panic!("Invalid addressing mode {:?}", mode);
            }
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let previous_carry = self.status & CARRY_FLAG;
        let res = self.a as u16 + value as u16 + previous_carry as u16;

        let carry = res > 0xff;
        self.update_carry_flag(carry);

        let overflow = (self.a ^ res as u8) & (value ^ res as u8) & 0x80 != 0;
        self.update_overflow_flag(overflow);

        self.a = res as u8;
        self.update_zero_and_negative_flags(res as u8);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.a &= value;
        self.update_zero_and_negative_flags(self.a);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.a = value;
        self.update_zero_and_negative_flags(value);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.a);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.update_zero_and_negative_flags(self.x);
    }

    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // second LSB is the zero flag
        if result == 0 {
            self.status = self.status | ZERO_FLAG;
        } else {
            self.status = self.status & !ZERO_FLAG;
        }

        // MSB is negative flag
        // if the new value of a is negative, ensure that the negative flag is set
        if result & 0b1000_0000 != 0 {
            self.status = self.status | NEGATIVE_FLAG;
        } else {
            self.status = self.status & !NEGATIVE_FLAG;
        }
    }

    fn update_overflow_flag(&mut self, overflow: bool) {
        if overflow {
            self.status = self.status | OVERFLOW_FLAG;
        } else {
            self.status = self.status & !OVERFLOW_FLAG;
        }
    }

    fn update_carry_flag(&mut self, carry: bool) {
        if carry {
            self.status = self.status | CARRY_FLAG;
        } else {
            self.status = self.status & !CARRY_FLAG;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = OpCode::from_u8(self.mem_read(self.program_counter));
            self.program_counter += 1;

            match opcode.mnemonic {
                "ADC" => self.adc(&opcode.mode),
                "AND" => self.and(&opcode.mode),
                "LDA" => self.lda(&opcode.mode),
                "STA" => self.sta(&opcode.mode),
                "TAX" => self.tax(),
                "INX" => self.inx(),
                "BRK" => return,
                _ => unreachable!(),
            }

            self.program_counter += opcode.bytes as u16 - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adc_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x69, 0x05, 0x00]); // ADC #$05
        cpu.run();
        assert_eq!(cpu.a, 0x05);
    }

    #[test]
    fn test_adc_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x05);
        cpu.load_and_run(vec![0x6d, 0x10, 0x00, 0x00]); // ADC $0010
        cpu.run();
        assert_eq!(cpu.a, 0x05);
    }

    #[test]
    fn test_and_immediate() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x29, 0xaa, 0x00]);
        cpu.a = 0b1010_1010;
        cpu.run();
        assert_eq!(cpu.a, 0b1010_1010);
    }

    #[test]
    fn test_and_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0b1010_1010);
        cpu.load(vec![0x2d, 0x10, 0x00, 0x00]);
        cpu.a = 0b1010_1010;
        cpu.run();
        assert_eq!(cpu.a, 0b1010_1010);
    }

    #[test]
    fn test_lda_works_immediate() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0x05, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.a, 0x05);
        // check zero and negative flags aren't set
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0000);
        assert_eq!(cpu.status & 0b1000_0000, 0b0000_0000);
    }

    #[test]
    fn test_lda_works_zero() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0x00, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_lda_works_from_memory() {
        // zero page
        let mut cpu = CPU::new();
        cpu.mem_write(0x69, 0x42);
        let program = vec![0xa5, 0x69];
        cpu.load_and_run(program);

        assert_eq!(cpu.a, 0x42);

        // absolute
        let mut cpu = CPU::new();
        cpu.mem_write(0x69, 0x42);
        let program = vec![0xad, 0x69, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_sta_works() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x00, 42);
        let program = vec![0xa5, 0x00, 0x85, 0x69, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read(0x69), 42);
    }

    #[test]
    fn test_tax_works() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0x69, 0xaa, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.x, 0x69);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0000);
        assert_eq!(cpu.status & 0b1000_0000, 0b0000_0000);
    }

    #[test]
    fn test_inx_works() {
        let mut cpu = CPU::new();
        let program = vec![0xe8, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.x, 0x01);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0000);
        assert_eq!(cpu.status & 0b1000_0000, 0b0000_0000);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        let mut program = vec![0xe8; 256];
        program.push(0x00);
        cpu.load_and_run(program);

        assert_eq!(cpu.x, 0)
    }
}

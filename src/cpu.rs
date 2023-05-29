pub enum OpCode {
    LDA,
    TAX,
    INX,
    BRK,
}

impl OpCode {
    pub fn from_u8(val: u8) -> OpCode {
        match val {
            0xa9 => OpCode::LDA,
            0xaa => OpCode::TAX,
            0xe8 => OpCode::INX,
            0x00 => OpCode::BRK,
            _ => panic!("Unkown opcode"),
        }
    }
}

pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xffff],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
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
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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

    fn lda(&mut self, value: u8) {
        self.a = value;
        self.update_zero_and_negative_flags(value);
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
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // MSB is negative flag
        // if the new value of a is negative, ensure that the negative flag is set
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = OpCode::from_u8(self.mem_read(self.program_counter));
            self.program_counter += 1;

            match opcode {
                OpCode::LDA => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(param);
                }
                OpCode::TAX => self.tax(),
                OpCode::INX => self.inx(),
                OpCode::BRK => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

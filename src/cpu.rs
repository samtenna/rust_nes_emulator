pub enum OpCode {
    LDA,
    TAX,
    BRK,
}

impl OpCode {
    pub fn from_u8(val: u8) -> OpCode {
        match val {
            0xa9 => OpCode::LDA,
            0xaa => OpCode::TAX,
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
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    fn lda(&mut self, value: u8) {
        self.a = value;
        self.update_zero_and_negative_flags(value);
    }

    fn tax(&mut self) {
        self.x = self.a;
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

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opcode = OpCode::from_u8(program[self.program_counter as usize]);
            self.program_counter += 1;

            match opcode {
                OpCode::LDA => {
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.lda(param);
                }
                OpCode::TAX => self.tax(),
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
        cpu.interpret(program);

        assert_eq!(cpu.a, 0x05);
        // check zero and negative flags aren't set
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0000);
        assert_eq!(cpu.status & 0b1000_0000, 0b0000_0000);
    }

    #[test]
    fn test_lda_works_zero() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0x00, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_tax_works() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0x69, 0xaa, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.x, 0x69);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0000);
        assert_eq!(cpu.status & 0b1000_0000, 0b0000_0000);
    }
}

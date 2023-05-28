pub enum OpCode {
    LDA,
    BRK,
}

impl OpCode {
    pub fn from_u8(val: u8) -> OpCode {
        match val {
            0xA9 => OpCode::LDA,
            0x00 => OpCode::BRK,
            _ => panic!("Unkown opcode"),
        }
    }
}

pub struct CPU {
    pub a: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            status: 0,
            program_counter: 0,
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
                    self.a = param;

                    // second LSB is the zero flag
                    if self.a == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    // MSB is negative flag
                    // if the new value of a is negative, ensure that the negative flag is set
                    if self.a & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }
                OpCode::BRK => {
                    return;
                }
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
        assert_eq!(cpu.status & 0b00_0010, 0b0000_0010);
    }
}

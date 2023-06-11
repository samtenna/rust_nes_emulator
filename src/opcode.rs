use lazy_static::lazy_static;

use crate::cpu::AddressingMode;

#[derive(PartialEq, Debug)]
pub struct OpCode {
    pub hex: u8,
    pub mnemonic: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

lazy_static! {
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
        // ADC
        OpCode::new(0x69, "ADC", 2, 3, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x6d, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, "ADC", 3, 4 /* +1 if page crossed */, AddressingMode::AbsoluteY),
        OpCode::new(0x79, "ADC", 3, 4 /* +1 if page crossed */, AddressingMode::AbsoluteY),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x71, "ADC", 2, 5 /* +2 if page crossed */, AddressingMode::IndirectY),
        // LDA
        OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA", 3, 4 /* +1 if page crossed */, AddressingMode::AbsoluteX),
        OpCode::new(0xb9, "LDA", 3, 4 /* +1 if page crossed */, AddressingMode::AbsoluteY),
        OpCode::new(0xa1, "LDA", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xb1, "LDA", 2, 5 /* +1 if page crossed */, AddressingMode::IndirectX),
        // STA
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA", 3, 5, AddressingMode::AbsoluteX),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::IndirectY),
        // TAX
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        // INX
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
        // BRK
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
    ];
}

impl OpCode {
    pub const fn new(
        hex: u8,
        mnemonic: &'static str,
        bytes: u8,
        cycles: u8,
        mode: AddressingMode,
    ) -> OpCode {
        OpCode {
            hex,
            mnemonic,
            bytes,
            cycles,
            mode,
        }
    }

    pub fn from_u8(val: u8) -> &'static OpCode {
        for op in CPU_OP_CODES.iter() {
            if val == op.hex {
                return op;
            }
        }

        panic!("Invalid opcode");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u8_works() {
        let opcode = OpCode::from_u8(0xa5);
        assert_eq!(
            *opcode,
            OpCode {
                hex: 0xa5,
                mnemonic: "LDA",
                bytes: 2,
                cycles: 3,
                mode: AddressingMode::ZeroPage,
            }
        );
    }
}

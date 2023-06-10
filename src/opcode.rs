use lazy_static::lazy_static;

use crate::cpu::AddressingMode;

pub struct OpCode {
    pub hex: u8,
    pub mnemonic: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

lazy_static! {
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
        // LDA
        OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        //
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
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

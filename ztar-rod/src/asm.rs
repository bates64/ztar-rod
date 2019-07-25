mod decode;
mod encode;
mod instructions;

pub use instructions::{Coprocessor, Instruction, Register};

impl Instruction {
    pub fn decode(instruction: [u8; 4]) -> Option<Instruction> {
        decode::decode(instruction)
    }
    pub fn encode(self) -> [u8; 4] {
        encode::encode(self)
    }
}

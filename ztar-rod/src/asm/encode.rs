use super::instructions::{Instruction, Register};
use Instruction::*;

impl core::ops::Shl<usize> for Register {
    type Output = u32;
    fn shl(self, rhs: usize) -> u32 {
        (self as u32) << rhs
    }
}

pub fn encode(instruction: Instruction) -> [u8; 4] {
    let num: u32 = match instruction {
        ADD(rd, rs, rt) => 0b000000 << 26 | rs << 21 | rt << 16 | rd << 11 | 0b100000,
        ADDI(rt, rs, imm) => 0b001000 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        ADDIU(rt, rs, imm) => 0b001001 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        ADDU(rd, rs, rt) => 0b000000 << 26 | rs << 21 | rt << 16 | rd << 11 | 0b100001,
        AND(rd, rs, rt) => 0b000000 << 26 | rs << 21 | rt << 16 | rd << 11 | 0b100100,
        ANDI(rt, rs, imm) => 0b001100 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        BCzF(cop, off) => (0b010000 | cop as u32) << 26 | 0b0100000000 << 16 | (off as u16 as u32),
        BCzFL(cop, off) => (0b010000 | cop as u32) << 26 | 0b0100000010 << 16 | (off as u16 as u32),
        BCzT(cop, off) => (0b010000 | cop as u32) << 26 | 0b0100000001 << 16 | (off as u16 as u32),
        BCzTL(cop, off) => (0b010000 | cop as u32) << 26 | 0b0100000011 << 16 | (off as u16 as u32),
        BEQ(rs, rt, offset) => 0b000100 << 26 | rs << 21 | rt << 16 | (offset as u16 as u32),
        BEQL(rs, rt, offset) => 0b010100 << 26 | rs << 21 | rt << 16 | (offset as u16 as u32),
        BGEZ(rs, offset) => 0b000001 << 26 | rs << 21 | 0b00001 << 16 | (offset as u16 as u32),
        BGEZAL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b10001 << 16 | (offset as u16 as u32),
        BGEZALL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b10011 << 16 | (offset as u16 as u32),
        BGEZL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b00011 << 16 | (offset as u16 as u32),
        BGTZ(rs, offset) => 0b000111 << 26 | rs << 21 | (offset as u16 as u32),
        BGTZL(rs, offset) => 0b010111 << 26 | rs << 21 | (offset as u16 as u32),
        BLEZ(rs, offset) => 0b000110 << 26 | rs << 21 | (offset as u16 as u32),
        BLEZL(rs, offset) => 0b010110 << 26 | rs << 21 | (offset as u16 as u32),
        BLTZ(rs, offset) => 0b000001 << 26 | rs << 21 | (offset as u16 as u32),
        BLTZAL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b10000 << 16 | (offset as u16 as u32),
        BLTZALL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b10010 << 16 | (offset as u16 as u32),
        BLTZL(rs, offset) => 0b000001 << 26 | rs << 21 | 0b00010 << 16 | (offset as u16 as u32),
        BNE(rs, rt, offset) => 0b000101 << 26 | rs << 21 | rt << 16 | (offset as u16 as u32),
        BNEL(rs, rt, offset) => 0b010101 << 26 | rs << 21 | rt << 16 | (offset as u16 as u32),
        BREAK(code) => (code & 0x000f_ffff) << 6 | 0b001101,
        CACHE(op, offset, base) => {
            0b101111 << 26 | base << 21 | (op as u32 & 0x1f) << 16 | (offset as u16 as u32)
        }
        CFCz(cop, rt, rd) => (0b010000 | cop as u32) << 26 | 0b00010 << 21 | rt << 16 | rd << 11,
        COPz(cop, cofun) => (0b010000 | cop as u32) << 26 | 0b1 << 25 | cofun & 0x01ff_ffff,
        CTCz(cop, rt, rd) => (0b010000 | cop as u32) << 26 | 0b00110 << 21 | rt << 16 | rd << 11,
        DADD(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101100,
        DADDI(rt, rs, imm) => 0b011000 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        DADDIU(rt, rs, imm) => 0b011001 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        DADDU(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101101,
        DDIV(rs, rt) => rs << 21 | rt << 16 | 0b011110,
        DDIVU(rs, rt) => rs << 21 | rt << 16 | 0b011111,
        DIV(rs, rt) => rs << 21 | rt << 16 | 0b011010,
        DIVU(rs, rt) => rs << 21 | rt << 16 | 0b011011,
        DMFC0(rt, rd) => 0b010000 << 26 | 0b00001 << 21 | rt << 16 | rd << 11,
        DMTC0(rt, rd) => 0b010000 << 26 | 0b00101 << 21 | rt << 16 | rd << 11,
        DMULT(rs, rt) => rs << 21 | rt << 16 | 0b011100,
        DMULTU(rs, rt) => rs << 21 | rt << 16 | 0b011101,
        DSLL(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111000,
        DSLLV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b010100,
        DSLL32(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111100,
        DSRA(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111011,
        DSRAV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b010111,
        DSRA32(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111111,
        DSRL(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111010,
        DSRLV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b010110,
        DSRL32(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b111110,
        DSUB(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101110,
        DSUBU(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101111,
        ERET => 0b010000 << 26 | 1 << 25 | 0b011000,
        J(target) => 0b000010 << 26 | (target >> 2) & 0x03ff_ffff,
        JAL(target) => 0b000011 << 26 | (target >> 2) & 0x03ff_ffff,
        JALR(rd, rs) => rs << 21 | rd << 11 | 0b001001,
        JR(rs) => rs << 21 | 0b001000,
        LB(rt, offset, base) => 0b100000 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LBU(rt, offset, base) => 0b100100 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LD(rt, offset, base) => 0b110111 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LDCz(cop, rt, offset, base) => {
            (0b110100 | cop as u32) << 26 | base << 21 | rt << 16 | (offset as u16 as u32)
        }
        LDL(rt, offset, base) => 0b011010 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LDR(rt, offset, base) => 0b011011 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LH(rt, offset, base) => 0b100001 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LHU(rt, offset, base) => 0b100101 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LL(rt, offset, base) => 0b110000 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LLD(rt, offset, base) => 0b110100 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LUI(rt, imm) => 0b001111 << 26 | rt << 16 | (imm as u16 as u32),
        LW(rt, offset, base) => 0b100011 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LWCz(cop, rt, offset, base) => {
            (0b110000 | cop as u32) << 26 | base << 21 | rt << 16 | (offset as u16 as u32)
        }
        LWL(rt, offset, base) => 0b100010 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LWR(rt, offset, base) => 0b100110 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        LWU(rt, offset, base) => 0b100111 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        MFC0(rt, rd) => 0b010000 << 26 | rt << 16 | rd << 11,
        MFCz(cop, rt, rd) => (0b010000 | cop as u32) << 26 | rt << 16 | rd << 11,
        MFHI(rd) => rd << 11 | 0b010000,
        MFLO(rd) => rd << 11 | 0b010010,
        MTC0(rt, rd) => 0b010000 << 26 | 0b00100 << 21 | rt << 16 | rd << 11,
        MTCz(cop, rt, rd) => (0b010000 | cop as u32) << 26 | 0b00100 << 21 | rt << 16 | rd << 11,
        MTHI(rs) => rs << 21 | 0b010001,
        MTLO(rs) => rs << 21 | 0b010011,
        MULT(rs, rt) => rs << 21 | rt << 16 | 0b011000,
        MULTU(rs, rt) => rs << 21 | rt << 16 | 0b011001,
        NOR(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b100111,
        OR(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b100101,
        ORI(rt, rs, imm) => 0b001101 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        SB(rt, offset, base) => 0b101000 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SC(rt, offset, base) => 0b111000 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SCD(rt, offset, base) => 0b111100 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SD(rt, offset, base) => 0b111111 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SDCz(cop, rt, offset, base) => {
            (0b111100 | cop as u32) << 26 | base << 21 | rt << 16 | (offset as u16 as u32)
        }
        SDL(rt, offset, base) => 0b101100 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SDR(rt, offset, base) => 0b101101 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SH(rt, offset, base) => 0b101001 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SLL(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6,
        SLLV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b000100,
        SLT(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101010,
        SLTI(rt, rs, imm) => 0b001010 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        SLTIU(rt, rs, imm) => 0b001011 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
        SLTU(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b101011,
        SRA(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b000011,
        SRAV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b000111,
        SRL(rd, rt, sa) => rt << 16 | rd << 11 | (sa as u32) << 6 | 0b000010,
        SRLV(rd, rt, rs) => rs << 21 | rt << 16 | rd << 11 | 0b000110,
        SUB(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b100010,
        SUBU(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b100011,
        SW(rt, offset, base) => 0b101011 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SWCz(cop, rt, offset, base) => {
            (0b111000 | cop as u32) << 26 | base << 21 | rt << 16 | (offset as u16 as u32)
        }
        SWL(rt, offset, base) => 0b101010 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SWR(rt, offset, base) => 0b101110 << 26 | base << 21 | rt << 16 | (offset as u16 as u32),
        SYNC => 0b001111,
        SYSCALL(code) => (code & 0x000f_ffff) << 6 | 0b001100,
        TEQ(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110100,
        TEQI(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01100 << 16 | (imm as u16 as u32),
        TGE(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110000,
        TGEI(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01000 << 16 | (imm as u16 as u32),
        TGEIU(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01001 << 16 | (imm as u16 as u32),
        TGEU(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110001,
        TLBP => 0b010000 << 26 | 1 << 25 | 0b001000,
        TLBR => 0b010000 << 26 | 1 << 25 | 0b000001,
        TLBWI => 0b010000 << 26 | 1 << 25 | 0b000010,
        TLBWR => 0b010000 << 26 | 1 << 25 | 0b000110,
        TLT(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110010,
        TLTI(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01010 << 16 | (imm as u16 as u32),
        TLTIU(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01011 << 16 | (imm as u16 as u32),
        TLTU(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110011,
        TNE(rs, rt, code) => rs << 21 | rt << 16 | (code as u32 & 0x03ff) << 6 | 0b110110,
        TNEI(rs, imm) => 0b000001 << 26 | rs << 21 | 0b01110 << 16 | (imm as u16 as u32),
        XOR(rd, rs, rt) => rs << 21 | rt << 16 | rd << 11 | 0b100110,
        XORI(rt, rs, imm) => 0b001110 << 26 | rs << 21 | rt << 16 | (imm as u16 as u32),
    };

    num.to_be_bytes()
}

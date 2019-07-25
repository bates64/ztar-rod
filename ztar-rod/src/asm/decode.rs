use core::convert::TryFrom;

use super::instructions::{Coprocessor, Instruction, Register};
use Instruction::*;

pub fn decode(instruction: [u8; 4]) -> Option<Instruction> {
    let instruction = u32::from_be_bytes(instruction);

    let opcode = instruction >> 26;
    let rs = Register::try_from((instruction >> 21) & 0x1f).unwrap();
    let rt = Register::try_from((instruction >> 16) & 0x1f).unwrap();
    let rd = Register::try_from((instruction >> 11) & 0x1f).unwrap();
    let sa = ((instruction >> 6) & 0x1f) as u8;
    let funct = instruction & 0x3f;

    let cop = || Coprocessor::try_from(opcode & 0b11).unwrap();
    let immediate = instruction as u16 as i16;
    let code10 = ((instruction >> 6) & 0x0000_03ff) as u16;
    let code20 = (instruction >> 6) & 0x000f_ffff;

    let rs0 = rs as u8 == 0;
    let rt0 = rt as u8 == 0;
    let rd0 = rd as u8 == 0;
    let sa0 = sa == 0;

    let (instruction, conditions) = match opcode {
        0b000000 => match funct {
            0b000000 => (SLL(rd, rt, sa), rs0),
            0b000010 => (SRL(rd, rt, sa), rs0),
            0b000011 => (SRA(rd, rt, sa), rs0),
            0b000100 => (SLLV(rd, rt, rs), sa0),
            0b000110 => (SRLV(rd, rt, rs), sa0),
            0b000111 => (SRAV(rd, rt, rs), sa0),
            0b001000 => (JR(rs), rt0 && rd0 && sa0),
            0b001001 => (JALR(rd, rs), rt0 && sa0),
            0b001100 => (SYSCALL(code20), true),
            0b001101 => (BREAK(code20), true),
            0b001111 => (SYNC, rs0 && rt0 && rd0 && sa0),
            0b010000 => (MFHI(rd), rs0 && rt0 && sa0),
            0b010001 => (MTHI(rs), rt0 && rd0 && sa0),
            0b010010 => (MFLO(rd), rs0 && rt0 && sa0),
            0b010011 => (MTLO(rs), rt0 && rd0 && sa0),
            0b010100 => (DSLLV(rd, rt, rs), sa0),
            0b010110 => (DSRLV(rd, rt, rs), sa0),
            0b010111 => (DSRAV(rd, rt, rs), sa0),
            0b011000 => (MULT(rs, rt), rd0 && sa0),
            0b011001 => (MULTU(rs, rt), rd0 && sa0),
            0b011010 => (DIV(rs, rt), rd0 && sa0),
            0b011011 => (DIVU(rs, rt), rd0 && sa0),
            0b011100 => (DMULT(rs, rt), rd0 && sa0),
            0b011101 => (DMULTU(rs, rt), rd0 && sa0),
            0b011110 => (DDIV(rs, rt), rd0 && sa0),
            0b011111 => (DDIVU(rs, rt), rd0 && sa0),
            0b100000 => (ADD(rd, rs, rt), sa0),
            0b100001 => (ADDU(rd, rs, rt), sa0),
            0b100010 => (SUB(rd, rs, rt), sa0),
            0b100011 => (SUBU(rd, rs, rt), sa0),
            0b100100 => (AND(rd, rs, rt), sa0),
            0b100101 => (OR(rd, rs, rt), sa0),
            0b100110 => (XOR(rd, rs, rt), sa0),
            0b100111 => (NOR(rd, rs, rt), sa0),
            0b101010 => (SLT(rd, rs, rt), sa0),
            0b101011 => (SLTU(rd, rs, rt), sa0),
            0b101100 => (DADD(rd, rs, rt), sa0),
            0b101101 => (DADDU(rd, rs, rt), sa0),
            0b101110 => (DSUB(rd, rs, rt), sa0),
            0b101111 => (DSUBU(rd, rs, rt), sa0),
            0b110000 => (TGE(rs, rt, code10), true),
            0b110001 => (TGEU(rs, rt, code10), true),
            0b110010 => (TLT(rs, rt, code10), true),
            0b110011 => (TLTU(rs, rt, code10), true),
            0b110100 => (TEQ(rs, rt, code10), true),
            0b110110 => (TNE(rs, rt, code10), true),
            0b111000 => (DSLL(rd, rt, sa), rs0),
            0b111010 => (DSRL(rd, rt, sa), rs0),
            0b111011 => (DSRA(rd, rt, sa), rs0),
            0b111100 => (DSLL32(rd, rt, sa), rs0),
            0b111110 => (DSRL32(rd, rt, sa), rs0),
            0b111111 => (DSRA32(rd, rt, sa), rs0),
            _ => return None,
        },

        0b000001 => match rt as u8 {
            0b00000 => (BLTZ(rs, immediate), true),
            0b00001 => (BGEZ(rs, immediate), true),
            0b00010 => (BLTZL(rs, immediate), true),
            0b00011 => (BGEZL(rs, immediate), true),
            0b01000 => (TGEI(rs, immediate), true),
            0b01001 => (TGEIU(rs, immediate), true),
            0b01010 => (TLTI(rs, immediate), true),
            0b01011 => (TLTIU(rs, immediate), true),
            0b01100 => (TEQI(rs, immediate), true),
            0b01110 => (TNEI(rs, immediate), true),
            0b10000 => (BLTZAL(rs, immediate), true),
            0b10001 => (BGEZAL(rs, immediate), true),
            0b10010 => (BLTZALL(rs, immediate), true),
            0b10011 => (BGEZALL(rs, immediate), true),
            _ => return None,
        },

        0b000010 => (J((instruction & 0x03ff_ffff) << 2), true),
        0b000011 => (JAL((instruction & 0x03ff_ffff) << 2), true),
        0b000100 => (BEQ(rs, rt, immediate), true),
        0b000101 => (BNE(rs, rt, immediate), true),
        0b000110 => (BLEZ(rs, immediate), rt0),
        0b000111 => (BGTZ(rs, immediate), rt0),
        0b001000 => (ADDI(rt, rs, immediate), true),
        0b001001 => (ADDIU(rt, rs, immediate), true),
        0b001010 => (SLTI(rt, rs, immediate), true),
        0b001011 => (SLTIU(rt, rs, immediate), true),
        0b001100 => (ANDI(rt, rs, immediate), true),
        0b001101 => (ORI(rt, rs, immediate), true),
        0b001110 => (XORI(rt, rs, immediate), true),
        0b001111 => (LUI(rt, immediate), rs0),

        0b010000 => match (rs as u8, funct) {
            (0b00000, 0b000000) => (MFC0(rt, rd), sa0),
            (0b00001, 0b000000) => (DMFC0(rt, rd), sa0),
            (0b00100, 0b000000) => (MTC0(rt, rd), sa0),
            (0b00101, 0b000000) => (DMTC0(rt, rd), sa0),
            (0b10000, 0b000001) => (TLBR, rt0 && rd0 && sa0),
            (0b10000, 0b000010) => (TLBWI, rt0 && rd0 && sa0),
            (0b10000, 0b000110) => (TLBWR, rt0 && rd0 && sa0),
            (0b10000, 0b001000) => (TLBP, rt0 && rd0 && sa0),
            (0b10000, 0b011000) => (ERET, rt0 && rd0 && sa0),
            _ => return None,
        },

        0b010001 | 0b010010 => match (rs as u8, rt as u8) {
            (0b00000, _) => (MFCz(cop(), rt, rd), sa0 && funct == 0),
            (0b00010, _) => (CFCz(cop(), rt, rd), sa0 && funct == 0),
            (0b00100, _) => (MTCz(cop(), rt, rd), sa0 && funct == 0),
            (0b00110, _) => (CTCz(cop(), rt, rd), sa0 && funct == 0),
            (0b01000, 0b00000) => (BCzF(cop(), immediate), true),
            (0b01000, 0b00001) => (BCzT(cop(), immediate), true),
            (0b01000, 0b00010) => (BCzFL(cop(), immediate), true),
            (0b01000, 0b00011) => (BCzTL(cop(), immediate), true),
            (rs, _) => (COPz(cop(), instruction & 0x01ff_ffff), rs & 0b10000 != 0),
        },

        0b010100 => (BEQL(rs, rt, immediate), true),
        0b010101 => (BNEL(rs, rt, immediate), true),
        0b010110 => (BLEZL(rs, immediate), rt0),
        0b010111 => (BGTZL(rs, immediate), rt0),
        0b011000 => (DADDI(rt, rs, immediate), true),
        0b011001 => (DADDIU(rt, rs, immediate), true),
        0b011010 => (LDL(rt, immediate, rs), true),
        0b011011 => (LDR(rt, immediate, rs), true),
        0b100000 => (LB(rt, immediate, rs), true),
        0b100001 => (LH(rt, immediate, rs), true),
        0b100010 => (LWL(rt, immediate, rs), true),
        0b100011 => (LW(rt, immediate, rs), true),
        0b100100 => (LBU(rt, immediate, rs), true),
        0b100101 => (LHU(rt, immediate, rs), true),
        0b100110 => (LWR(rt, immediate, rs), true),
        0b100111 => (LWU(rt, immediate, rs), true),
        0b101000 => (SB(rt, immediate, rs), true),
        0b101001 => (SH(rt, immediate, rs), true),
        0b101010 => (SWL(rt, immediate, rs), true),
        0b101011 => (SW(rt, immediate, rs), true),
        0b101100 => (SDL(rt, immediate, rs), true),
        0b101101 => (SDR(rt, immediate, rs), true),
        0b101110 => (SWR(rt, immediate, rs), true),
        0b101111 => (CACHE(rt as u8, immediate, rs), true),
        0b110000 => (LL(rt, immediate, rs), true),
        0b110001 | 0b110010 => (LWCz(cop(), rt, immediate, rs), true),
        0b110100 => (LLD(rt, immediate, rs), true),
        0b110101 | 0b110110 => (LDCz(cop(), rt, immediate, rs), true),
        0b110111 => (LD(rt, immediate, rs), true),
        0b111000 => (SC(rt, immediate, rs), true),
        0b111001 | 0b111010 => (SWCz(cop(), rt, immediate, rs), true),
        0b111100 => (SCD(rt, immediate, rs), true),
        0b111101 | 0b111110 => (SDCz(cop(), rt, immediate, rs), true),
        0b111111 => (SD(rt, immediate, rs), true),

        _ => return None,
    };

    if conditions {
        Some(instruction)
    } else {
        None
    }
}

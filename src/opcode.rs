use super::util::{concat_12, concat_8};

#[derive(Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    SYS(u16),
    CLS,
    RET,
    JP(u16),
    CALL(u16),
    SE(usize, u8),
    SNE(usize, u8),
    SE_V(usize, usize),
    LD(usize, u8),
    ADD(usize, u8),
    LD_V(usize, usize),
    OR(usize, usize),
    AND(usize, usize),
    XOR(usize, usize),
    ADD_V(usize, usize),
    SUB(usize, usize),
    SHR(usize, usize),
    SUBN(usize, usize),
    SHL(usize, usize),
    SNE_V(usize, usize),
    LD_I(u16),
    JP_V(u16),
    RND(usize, u8),
    DRW(usize, usize, u8),
    SKP(usize),
    SKNP(usize),
    LD_V_DT(usize),
    LD_K(usize),
    LD_DT_V(usize),
    LD_ST(usize),
    ADD_I(usize),
    LD_F(usize),
    LD_B(usize),
    LD_I_V(usize),
    LD_V_I(usize),
    UNDEFINED
}


impl Opcode {

    pub fn from_bytes(bytes: u16) -> Opcode {

        let a = ((bytes & 0xF000) >> 12) as u8;
        let b = ((bytes & 0x0F00) >> 8) as u8;
        let c = ((bytes & 0x00F0) >> 4) as u8;
        let d = (bytes & 0x000F) as u8;


        match (a,b,c,d) {
            (0x0, 0x0, 0xE, 0x0) => Opcode::CLS,
            (0x0, 0x0, 0xE, 0xE) => Opcode::RET,
            (0x0, x, y, z) => Opcode::SYS(concat_12(x,y,z)),
            (0x1, x, y, z) => Opcode::JP(concat_12(x,y,z)),
            (0x2, x, y, z) => Opcode::CALL(concat_12(x,y,z)),
            (0x3, x, y, z) => Opcode::SE(x as usize, concat_8(y,z)),
            (0x4, x, y, z) => Opcode::SNE(x as usize, concat_8(y,z)),
            (0x5, x, y, 0x0) => Opcode::SE_V(x as usize, y as usize),
            (0x6, x, y, z) => Opcode::LD(x as usize, concat_8(y,z)),
            (0x7, x, y, z) => Opcode::ADD(x as usize, concat_8(y,z)),
            (0x8, x, y, 0x0) => Opcode::LD_V(x as usize, y as usize),
            (0x8, x, y, 0x1) => Opcode::OR(x as usize, y as usize),
            (0x8, x, y, 0x2) => Opcode::AND(x as usize, y as usize),
            (0x8, x, y, 0x3) => Opcode::XOR(x as usize, y as usize),
            (0x8, x, y, 0x4) => Opcode::ADD_V(x as usize, y as usize),
            (0x8, x, y, 0x5) => Opcode::SUB(x as usize, y as usize),
            (0x8, x, y, 0x6) => Opcode::SHR(x as usize, y as usize),
            (0x8, x, y, 0x7) => Opcode::SUBN(x as usize, y as usize),
            (0x8, x, y, 0xE) => Opcode::SHL(x as usize, y as usize),
            (0x9, x, y, 0x0) => Opcode::SNE_V(x as usize, y as usize),
            (0xA, x, y, z) => Opcode::LD_I(concat_12(x,y,z)),
            (0xB, x, y, z) => Opcode::JP_V(concat_12(x,y,z)),
            (0xC, x, y, z) => Opcode::RND(x as usize, concat_8(y,z)),
            (0xD, x, y, z) => Opcode::DRW(x as usize, y as usize, z),
            (0xE, x, 0x9, 0xE) => Opcode::SKP(x as usize),
            (0xE, x, 0xA, 0x1) => Opcode::SKNP(x as usize),
            (0xF, x, 0x0, 0x7) => Opcode::LD_V_DT(x as usize),
            (0xF, x, 0x0, 0xA) => Opcode::LD_K(x as usize),
            (0xF, x, 0x1, 0x5) => Opcode::LD_DT_V(x as usize),
            (0xF, x, 0x1, 0x8) => Opcode::LD_ST(x as usize),
            (0xF, x, 0x1, 0xE) => Opcode::ADD_I(x as usize),
            (0xF, x, 0x2, 0x9) => Opcode::LD_F(x as usize),
            (0xF, x, 0x3, 0x3) => Opcode::LD_B(x as usize),
            (0xF, x, 0x5, 0x5) => Opcode::LD_I_V(x as usize),
            (0xF, x, 0x6, 0x5) => Opcode::LD_V_I(x as usize),
            _ => Opcode::UNDEFINED
        }
    }
}

#[test]
fn test_nonexistant_opcode() {
    let code = 0xFFFF;
    assert_eq!(Opcode::from_bytes(code), Opcode::UNDEFINED);
}

#[test]
fn test_constant_opcode() {
    let code = 0x00EE;
    assert_eq!(Opcode::from_bytes(code), Opcode::RET);
}

#[test]
fn test_single_variable_opcode() {
    let code =0xF565;
    assert_eq!(Opcode::from_bytes(code), Opcode::LD_V_I(5));
}

#[test]
fn test_double_variable_opcode() {
    let code = 0x8AB1;
    assert_eq!(Opcode::from_bytes(code), Opcode::OR(0xA, 0xB));
}

#[test]
fn test_triplet_variable_opcode() {
    let code = 0x0AF2;
    assert_eq!(Opcode::from_bytes(code), Opcode::SYS(0xAF2));
}

#[test]
fn test_single_double_variable_opcode() {
    let code = 0x72FE;
    assert_eq!(Opcode::from_bytes(code), Opcode::ADD(0x2, 0xFE));
}
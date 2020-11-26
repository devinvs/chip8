pub fn concat_12(x: u8, y: u8, z: u8) -> u16 {
    ((x as u16) << 8) | ((y as u16) << 4) | (z as u16)
}

#[test]
fn test_concat_12() {
    let a = 0b1010;
    let b = 0b0010;
    let c = 0b1101;

    let concat = concat_12(a,b,c);

    assert_eq!(concat, 0b101000101101);
}


pub fn concat_8(y: u8, z: u8) -> u8 {
    y << 4 | z
}

#[test]
fn test_concat_8() {
    let a = 0b1011;
    let b = 0b0100;

    assert_eq!(concat_8(a,b), 0b10110100);
}


pub fn byte_to_bits(byte: u8) -> [u8; 8] {
    let mut bits = [0; 8];
    bits[0] = (byte & 0b10000000) >> 7;
    bits[1] = (byte & 0b01000000) >> 6;
    bits[2] = (byte & 0b00100000) >> 5;
    bits[3] = (byte & 0b00010000) >> 4;
    bits[4] = (byte & 0b00001000) >> 3;
    bits[5] = (byte & 0b00000100) >> 2;
    bits[6] = (byte & 0b00000010) >> 1;
    bits[7] = byte & 0b00000001;

    bits
}

#[test]
fn test_byte_to_bits() {
    let byte = 0xF4;
    let bits = byte_to_bits(byte);

    assert_eq!(bits, [1,1,1,1,0,1,0,0]);
}
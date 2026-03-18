pub fn clamp(byte: u8) -> u8 {
    byte & 248
}

pub fn mask_low(byte: u8) -> u8 {
    byte & 127
}

pub fn process(input: u8) -> u8 {
    mask_low(clamp(input))
}

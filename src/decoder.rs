pub fn decode_data(data: &mut Vec<u8>, key1: &Vec<u8>, key2: &Vec<u8>) {
    let mut xor_sum = (data.len() + 120) as u16;

    for c in data {
        *c ^= key1[xor_sum as usize] ^ key2[(((xor_sum as usize) + 120) as u8) as usize];
        xor_sum = xor_sum.wrapping_add(key2[(xor_sum as u8) as usize] as u16);
    }
}

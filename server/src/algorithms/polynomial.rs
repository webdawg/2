pub fn generate(a: u32, b: u32, c: u32, coordinate: u64, length: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(length as usize);
    for i in 0..length as u64 {
        let x = coordinate.wrapping_add(i);
        let val = (a as u64)
            .wrapping_mul(x)
            .wrapping_mul(x)
            .wrapping_add((b as u64).wrapping_mul(x))
            .wrapping_add(c as u64);
        out.push(val as u8);
    }
    out
}

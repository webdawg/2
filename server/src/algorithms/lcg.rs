const MULT: u64 = 6364136223846793005;
const INC: u64 = 1442695040888963407;

// Closed-form jump-ahead so a coordinate can be reached in O(log n) steps
// instead of replaying the generator from the start.
fn advance(state: u64, mut mult: u64, mut plus: u64, mut n: u64) -> u64 {
    let mut acc_mult: u64 = 1;
    let mut acc_plus: u64 = 0;
    while n > 0 {
        if n & 1 == 1 {
            acc_mult = acc_mult.wrapping_mul(mult);
            acc_plus = acc_plus.wrapping_mul(mult).wrapping_add(plus);
        }
        plus = plus.wrapping_mul(mult.wrapping_add(1));
        mult = mult.wrapping_mul(mult);
        n >>= 1;
    }
    acc_mult.wrapping_mul(state).wrapping_add(acc_plus)
}

pub fn generate(seed: u64, coordinate: u64, length: u32) -> Vec<u8> {
    let mut state = advance(seed, MULT, INC, coordinate);
    let mut out = Vec::with_capacity(length as usize);
    for _ in 0..length {
        out.push((state >> 56) as u8);
        state = state.wrapping_mul(MULT).wrapping_add(INC);
    }
    out
}

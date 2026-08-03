use sha2::{Digest, Sha256};

const HASH_LEN: u64 = 32;

// Each block only exists as the hash of the previous one, so reaching a given
// coordinate means recomputing every block from 0 up to it — O(coordinate),
// unlike the LCG's jump-ahead. Fine for the small-scale search prototype; a
// large-coordinate scheme would need a different construction.
pub fn generate(seed: &[u8], coordinate: u64, length: u32) -> Vec<u8> {
    if length == 0 {
        return Vec::new();
    }

    let start_block = coordinate / HASH_LEN;
    let start_offset = (coordinate % HASH_LEN) as usize;
    let end_pos = coordinate + length as u64;
    let end_block = (end_pos - 1) / HASH_LEN;

    let mut out = Vec::with_capacity(length as usize);
    let mut hash = Sha256::digest(seed);
    let mut block_idx = 0u64;
    loop {
        if block_idx >= start_block && block_idx <= end_block {
            let bytes = hash.as_slice();
            let from = if block_idx == start_block { start_offset } else { 0 };
            let to = if block_idx == end_block {
                ((end_pos - 1) % HASH_LEN) as usize + 1
            } else {
                HASH_LEN as usize
            };
            out.extend_from_slice(&bytes[from..to]);
        }
        if block_idx == end_block {
            break;
        }
        hash = Sha256::digest(hash.as_slice());
        block_idx += 1;
    }
    out
}

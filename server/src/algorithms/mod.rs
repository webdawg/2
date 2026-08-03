mod hash_chain;
mod lcg;
mod polynomial;

use std::fmt;

pub const ALGO_LCG: u8 = 0;
pub const ALGO_HASH_CHAIN: u8 = 1;
pub const ALGO_POLYNOMIAL: u8 = 2;

#[derive(Debug)]
pub enum GenError {
    UnknownAlgorithm(u8),
    InvalidParams(&'static str),
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenError::UnknownAlgorithm(id) => write!(f, "unknown algorithm id {id}"),
            GenError::InvalidParams(msg) => write!(f, "invalid params: {msg}"),
        }
    }
}

pub fn generate(
    algorithm_id: u8,
    coordinate: u64,
    length: u32,
    params: &[u8],
) -> Result<Vec<u8>, GenError> {
    match algorithm_id {
        ALGO_LCG => {
            let seed_bytes: [u8; 8] = params
                .try_into()
                .map_err(|_| GenError::InvalidParams("lcg requires an 8-byte seed"))?;
            let seed = u64::from_be_bytes(seed_bytes);
            Ok(lcg::generate(seed, coordinate, length))
        }
        ALGO_HASH_CHAIN => {
            if params.is_empty() || params.len() > 256 {
                return Err(GenError::InvalidParams(
                    "hash-chain seed must be 1..=256 bytes",
                ));
            }
            Ok(hash_chain::generate(params, coordinate, length))
        }
        ALGO_POLYNOMIAL => {
            if params.len() != 12 {
                return Err(GenError::InvalidParams(
                    "polynomial requires 12 bytes (a, b, c as u32 BE)",
                ));
            }
            let a = u32::from_be_bytes(params[0..4].try_into().unwrap());
            let b = u32::from_be_bytes(params[4..8].try_into().unwrap());
            let c = u32::from_be_bytes(params[8..12].try_into().unwrap());
            Ok(polynomial::generate(a, b, c, coordinate, length))
        }
        other => Err(GenError::UnknownAlgorithm(other)),
    }
}

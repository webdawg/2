use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use server::algorithms::{ALGO_HASH_CHAIN, ALGO_LCG, ALGO_POLYNOMIAL};
use server::protocol::{read_response, Request, Response};
use std::env;
use std::fs;
use std::net::TcpStream;
use std::process;

struct Args {
    target: Vec<u8>,
    addr: String,
    attempts: u64,
    algorithms: Vec<u8>,
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: client --target <text|@file> [--addr host:port] [--attempts N] [--algorithms 0,1,2]"
    );
    process::exit(1);
}

fn parse_args() -> Args {
    let mut target: Option<Vec<u8>> = None;
    let mut addr = "127.0.0.1:7878".to_string();
    let mut attempts: u64 = 200_000;
    let mut algorithms = vec![ALGO_LCG, ALGO_HASH_CHAIN, ALGO_POLYNOMIAL];

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" => {
                let value = args.next().unwrap_or_else(|| print_usage_and_exit());
                target = Some(match value.strip_prefix('@') {
                    Some(path) => fs::read(path).unwrap_or_else(|e| {
                        eprintln!("failed to read {path}: {e}");
                        process::exit(1);
                    }),
                    None => value.into_bytes(),
                });
            }
            "--addr" => addr = args.next().unwrap_or_else(|| print_usage_and_exit()),
            "--attempts" => {
                attempts = args
                    .next()
                    .unwrap_or_else(|| print_usage_and_exit())
                    .parse()
                    .unwrap_or_else(|_| print_usage_and_exit());
            }
            "--algorithms" => {
                let value = args.next().unwrap_or_else(|| print_usage_and_exit());
                algorithms = value
                    .split(',')
                    .map(|s| s.parse().unwrap_or_else(|_| print_usage_and_exit()))
                    .collect();
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage_and_exit();
            }
        }
    }

    let target = target.unwrap_or_else(|| print_usage_and_exit());
    Args {
        target,
        addr,
        attempts,
        algorithms,
    }
}

// hash-chain has no jump-ahead (see server::algorithms::hash_chain): reaching
// coordinate N costs N/32 chained SHA-256 calls. A full random u64 coordinate
// would pick an astronomically large N almost every time and never return, so
// its search range is capped; lcg and polynomial have O(log n)/O(1) jump-ahead
// and can use the full range.
const MAX_HASH_CHAIN_COORDINATE: u64 = 1_000_000;

fn random_coordinate(algorithm_id: u8, rng: &mut StdRng) -> u64 {
    match algorithm_id {
        ALGO_HASH_CHAIN => rng.gen_range(0..=MAX_HASH_CHAIN_COORDINATE),
        _ => rng.gen(),
    }
}

fn random_params(algorithm_id: u8, rng: &mut StdRng) -> Vec<u8> {
    match algorithm_id {
        ALGO_LCG => rng.gen::<u64>().to_be_bytes().to_vec(),
        ALGO_HASH_CHAIN => {
            let len: usize = rng.gen_range(1..=32);
            (0..len).map(|_| rng.gen::<u8>()).collect()
        }
        ALGO_POLYNOMIAL => {
            let mut p = Vec::with_capacity(12);
            p.extend_from_slice(&rng.gen::<u32>().to_be_bytes());
            p.extend_from_slice(&rng.gen::<u32>().to_be_bytes());
            p.extend_from_slice(&rng.gen::<u32>().to_be_bytes());
            p
        }
        other => panic!("no param generator registered for algorithm {other}"),
    }
}

fn algorithm_name(id: u8) -> &'static str {
    match id {
        ALGO_LCG => "lcg",
        ALGO_HASH_CHAIN => "hash-chain",
        ALGO_POLYNOMIAL => "polynomial",
        _ => "unknown",
    }
}

// Matching bytes at the same position - a coarse similarity signal so a
// non-matching search still reports how close it got, rather than a bare "no".
fn score(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).filter(|(x, y)| x == y).count()
}

struct SearchResult {
    found: bool,
    attempts_used: u64,
    best_score: usize,
    coordinate: u64,
    params: Vec<u8>,
}

fn search(
    stream: &mut TcpStream,
    algorithm_id: u8,
    target: &[u8],
    attempts: u64,
    rng: &mut StdRng,
) -> std::io::Result<SearchResult> {
    let mut best_score = 0;
    let mut best_coordinate = 0u64;
    let mut best_params = Vec::new();

    for attempt in 1..=attempts {
        let coordinate = random_coordinate(algorithm_id, rng);
        let params = random_params(algorithm_id, rng);

        let req = Request {
            algorithm_id,
            coordinate,
            length: target.len() as u32,
            params: params.clone(),
        };
        req.write_to(stream)?;

        let data = match read_response(stream)? {
            Response::Ok(data) => data,
            Response::Err(_) => continue,
        };

        let s = score(&data, target);
        if s > best_score {
            best_score = s;
            best_coordinate = coordinate;
            best_params = params.clone();
        }
        if data == target {
            return Ok(SearchResult {
                found: true,
                attempts_used: attempt,
                best_score: target.len(),
                coordinate,
                params,
            });
        }
    }

    Ok(SearchResult {
        found: false,
        attempts_used: attempts,
        best_score,
        coordinate: best_coordinate,
        params: best_params,
    })
}

fn main() -> std::io::Result<()> {
    let args = parse_args();
    let mut rng = StdRng::from_entropy();

    println!(
        "searching for {} target byte(s) against {}",
        args.target.len(),
        args.addr
    );

    for &algorithm_id in &args.algorithms {
        let mut stream = TcpStream::connect(&args.addr)?;
        stream.set_nodelay(true)?;
        let result = search(&mut stream, algorithm_id, &args.target, args.attempts, &mut rng)?;

        if result.found {
            println!(
                "[{}] MATCH after {} attempts: coordinate={} params={:02x?}",
                algorithm_name(algorithm_id),
                result.attempts_used,
                result.coordinate,
                result.params
            );
        } else {
            println!(
                "[{}] no exact match in {} attempts; best {}/{} bytes matched (coordinate={} params={:02x?})",
                algorithm_name(algorithm_id),
                result.attempts_used,
                result.best_score,
                args.target.len(),
                result.coordinate,
                result.params
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_counts_matching_bytes_at_same_position() {
        assert_eq!(score(b"abcd", b"abXY"), 2);
        assert_eq!(score(b"", b""), 0);
        assert_eq!(score(b"abc", b"xyz"), 0);
    }

    #[test]
    fn random_params_has_expected_length_per_algorithm() {
        let mut rng = StdRng::seed_from_u64(1);
        assert_eq!(random_params(ALGO_LCG, &mut rng).len(), 8);
        assert_eq!(random_params(ALGO_POLYNOMIAL, &mut rng).len(), 12);
        let hash_params = random_params(ALGO_HASH_CHAIN, &mut rng);
        assert!(!hash_params.is_empty() && hash_params.len() <= 32);
    }

    #[test]
    fn hash_chain_coordinate_stays_within_tractable_bound() {
        let mut rng = StdRng::seed_from_u64(2);
        for _ in 0..1000 {
            let coordinate = random_coordinate(ALGO_HASH_CHAIN, &mut rng);
            assert!(coordinate <= MAX_HASH_CHAIN_COORDINATE);
        }
    }
}

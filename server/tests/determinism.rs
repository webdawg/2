use server::algorithms::{ALGO_HASH_CHAIN, ALGO_LCG, ALGO_POLYNOMIAL};
use server::protocol::{read_response, Request, Response};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

fn spawn_test_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let _ = server::run(listener);
    });
    addr
}

fn request(addr: SocketAddr, algorithm_id: u8, coordinate: u64, length: u32, params: Vec<u8>) -> Response {
    let mut stream = TcpStream::connect(addr).unwrap();
    let req = Request {
        algorithm_id,
        coordinate,
        length,
        params,
    };
    req.write_to(&mut stream).unwrap();
    read_response(&mut stream).unwrap()
}

#[test]
fn lcg_is_deterministic_across_connections() {
    let addr = spawn_test_server();
    let seed = 42u64.to_be_bytes().to_vec();
    let a = request(addr, ALGO_LCG, 100, 16, seed.clone());
    let b = request(addr, ALGO_LCG, 100, 16, seed);
    match (a, b) {
        (Response::Ok(x), Response::Ok(y)) => assert_eq!(x, y),
        other => panic!("expected Ok/Ok, got {other:?}"),
    }
}

#[test]
fn lcg_different_coordinates_differ() {
    let addr = spawn_test_server();
    let seed = 42u64.to_be_bytes().to_vec();
    let a = request(addr, ALGO_LCG, 0, 16, seed.clone());
    let b = request(addr, ALGO_LCG, 1000, 16, seed);
    match (a, b) {
        (Response::Ok(x), Response::Ok(y)) => assert_ne!(x, y),
        other => panic!("expected Ok/Ok, got {other:?}"),
    }
}

#[test]
fn hash_chain_is_deterministic_and_matches_contiguous_read() {
    let addr = spawn_test_server();
    let seed = b"test-seed".to_vec();
    let whole = request(addr, ALGO_HASH_CHAIN, 0, 40, seed.clone());
    let part1 = request(addr, ALGO_HASH_CHAIN, 0, 20, seed.clone());
    let part2 = request(addr, ALGO_HASH_CHAIN, 20, 20, seed);
    match (whole, part1, part2) {
        (Response::Ok(whole), Response::Ok(p1), Response::Ok(p2)) => {
            let mut combined = p1;
            combined.extend(p2);
            assert_eq!(whole, combined);
        }
        other => panic!("expected Ok responses, got {other:?}"),
    }
}

#[test]
fn polynomial_is_deterministic() {
    let addr = spawn_test_server();
    let mut params = Vec::new();
    params.extend_from_slice(&3u32.to_be_bytes());
    params.extend_from_slice(&5u32.to_be_bytes());
    params.extend_from_slice(&7u32.to_be_bytes());
    let a = request(addr, ALGO_POLYNOMIAL, 10, 8, params.clone());
    let b = request(addr, ALGO_POLYNOMIAL, 10, 8, params);
    match (a, b) {
        (Response::Ok(x), Response::Ok(y)) => assert_eq!(x, y),
        other => panic!("expected Ok/Ok, got {other:?}"),
    }
}

#[test]
fn unknown_algorithm_returns_error() {
    let addr = spawn_test_server();
    let resp = request(addr, 99, 0, 8, vec![]);
    match resp {
        Response::Err(_) => {}
        other => panic!("expected error, got {other:?}"),
    }
}

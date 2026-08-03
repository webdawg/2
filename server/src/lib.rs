pub mod algorithms;
pub mod protocol;

use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn run(listener: TcpListener) -> io::Result<()> {
    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {
            if let Err(e) = handle_connection(stream) {
                eprintln!("connection error: {e}");
            }
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    loop {
        let request = match protocol::read_request(&mut stream) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };

        let response = match algorithms::generate(
            request.algorithm_id,
            request.coordinate,
            request.length,
            &request.params,
        ) {
            Ok(data) => protocol::Response::Ok(data),
            Err(e) => protocol::Response::Err(e.to_string()),
        };

        response.write_to(&mut stream)?;
    }
}

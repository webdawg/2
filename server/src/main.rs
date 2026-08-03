use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let port = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(7878);

    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("listening on 127.0.0.1:{port}");
    server::run(listener)
}

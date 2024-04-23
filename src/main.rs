// Uncomment this block to pass the first stage
use std::{io::Write, net::TcpListener};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!(
                    "accepted new connection local:{} remote:{}",
                    _stream.local_addr().unwrap().ip(),
                    _stream.peer_addr().unwrap().ip()
                );
                let _ = _stream
                    .write(b"HTTP/1.1 200 OK\r\n\r\n")
                    .expect("Failed to write response buffer");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

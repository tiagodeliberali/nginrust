use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    let buffer_size = stream.read(&mut buffer).unwrap();

    let get_index = b"GET / HTTP/1.1\r\n";
    let get_slow = b"GET /slow HTTP/1.1\r\n";

    let (status_code, contents) = if buffer.starts_with(get_index) {
        let contents = fs::read_to_string("static/index.html").unwrap();
        ("HTTP/1.1 200 OK", contents)
    } else if buffer.starts_with(get_slow) {
        let contents = fs::read_to_string("static/slow.html").unwrap();
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", contents)
    } else {
        let contents = fs::read_to_string("static/404.html").unwrap();
        ("HTTP/1.1 404 NOT FOUND", contents)
    };

    println!("Request size: {}", buffer_size);
    println!("Request {}", String::from_utf8_lossy(&buffer[..]));

    let response = format!("{}\r\n\r\n{}", status_code, contents);

    let write_size = stream.write(response.as_bytes()).unwrap();
    println!("Response size: {}", write_size);
    stream.flush().unwrap();
}

use nginrust::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|worker_id: usize| {
            handle_connection(stream, worker_id);
        });
    }

    println!("[GLOBAL] Fechando o servidor...");
}

fn handle_connection(mut stream: TcpStream, worker_id: usize) {
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

    println!("[WORKER-{}] Request size: {}", worker_id, buffer_size);

    let response = format!("{}\r\n\r\n{}", status_code, contents);

    let write_size = stream.write(response.as_bytes()).unwrap();
    println!("[WORKER-{}] Response size: {}", worker_id, write_size);
    stream.flush().unwrap();
}

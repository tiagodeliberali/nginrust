extern crate ctrlc;
use nginrust::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = Arc::new(Mutex::new(ThreadPool::new(4)));
    let app_stoped = Arc::new(AtomicBool::new(false));

    shutdown_on_signal(&pool, &app_stoped);

    for stream in listener.incoming() {
        println!("[GLOBAL] Recebendo novo request");
        let stream = stream.unwrap();

        if app_stoped.load(Ordering::Relaxed) {
            handle_stoped_connection(stream);
            continue;
        }

        let result = pool.lock().unwrap().execute(|worker_id: usize| {
            handle_connection(stream, worker_id);
        });

        if let Err(msg) = result {
            println!("[GLOBAL] Error on thread {}", msg);
        }
    }
}

fn shutdown_on_signal(pool: &Arc<Mutex<ThreadPool>>, app_stoped: &Arc<AtomicBool>) {
    let clone_pool = Arc::clone(&pool);
    let clone_app_stoped = Arc::clone(&app_stoped);
    ctrlc::set_handler(move || {
        {
            clone_app_stoped.store(true, Ordering::Relaxed)
        }

        println!("\r\n[GLOBAL] Alguem apertou ctrl+c!");
        clone_pool.lock().unwrap().shut_down();
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
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
        thread::sleep(Duration::from_secs(10));
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

fn handle_stoped_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    let buffer_size = stream.read(&mut buffer).unwrap();

    println!(
        "[GLOBAL] Request recusada. Estamos encerrando. Size: {}",
        buffer_size
    );

    let response = "HTTP/1.1 503 SERVICE UNAVAILABLE\r\n\r\nServer shutting down";

    let write_size = stream.write(response.as_bytes()).unwrap();
    println!("[GLOBAL] Response size: {}", write_size);
    stream.flush().unwrap();
}

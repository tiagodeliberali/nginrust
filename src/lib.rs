use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::error;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    is_shut_down: bool,
}

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender, is_shut_down: false }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), Box<dyn error::Error>>
    where
        F: FnOnce(usize) + Send + 'static,
    {
        if self.is_shut_down {
            return Err(String::from("Servidor está desligando e não aceitamos mais requisições").into());
        }

        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();

        Ok(())
    }

    pub fn finish(&mut self) {
        self.is_shut_down = true;

        println!("[GLOBAL] Avisando todo mundo que é para parar...");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("[GLOBAL] Todas mensagens para parar enviadas");

        for worker in &mut self.workers {
            println!("[WORKER-{}] Encerrando o trabalho...", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
            println!("[WORKER-{}] Encerrado.", worker.id);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.finish();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("[WORKER-{}] Eu tenho um trabalho!", id);
                    job(id);
                }
                Message::Terminate => {
                    println!("[WORKER-{}] Vou terminar!", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

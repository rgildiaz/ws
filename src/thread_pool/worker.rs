use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};

pub struct Worker {
    pub id: usize,
    pub thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();
            match msg {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job()
                }
                Err(err) => {
                    println!("Worker {} got an error: {}", id, err);
                    break;
                }
            }
        });
        Worker { id, thread }
    }
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

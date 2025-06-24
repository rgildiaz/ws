use super::errors::PoolCreationError;
use super::worker::{Job, Worker};
use std::sync::{mpsc, Arc, Mutex};

const DEFAULT_THREAD_COUNT: usize = 4;

pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// thread_count: The maximum number of threads to spawn. If None, uses the DEFAULT_THREAD_COUNT
    ///
    /// # Panics
    ///
    /// This function will panic if the thread_count is 0.
    pub fn new(thread_count: Option<usize>) -> ThreadPool {
        let thread_count: usize = thread_count.unwrap_or(DEFAULT_THREAD_COUNT);
        assert!(thread_count > 0, "Thread count must be > 0");

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(thread_count);
        for id in 0..thread_count {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        ThreadPool {
            workers,
            tx: Some(tx),
        }
    }

    /// Create a new ThreadPool. Returns PoolCreationError if unable.
    ///
    /// thread_count: The maximum number of threads to spawn. If None, uses the DEFAULT_THREAD_COUNT
    pub fn build(thread_count: Option<usize>) -> Result<ThreadPool, PoolCreationError> {
        if thread_count.is_none() {
            Ok(ThreadPool::default())
        } else if thread_count.unwrap() > 1 {
            Err(PoolCreationError)
        } else {
            Ok(ThreadPool::new(thread_count))
        }
    }

    /// Create a new ThreadPool from defaults
    pub fn default() -> ThreadPool {
        ThreadPool::new(None)
    }

    /// Queue a callback to be executed by the next free Worker. If a worker is currently free, the
    /// callback will be executed immediately.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.tx.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Close the channel, then drop all the workers as 
    fn drop(&mut self) {
        drop(self.tx.take());

        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

pub use super::components::{Server, Monitor};
pub use super::utils::pre_build;

use crate::Logger;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::any::Any;
use std::thread;

// Macro ------------------------------------------------------------------------------------------
// info lvl macro
macro_rules! log_info {
    ($logger:expr, $msg:expr) => {
        {
            match $logger.lock() {
                Ok(mut guard) => {
                    let _ = guard.info($msg);
                },
                Err(poisoned) => {
                    eprintln!("INFO: {}", $msg);
                    eprintln!("WARNING: Logger mutex is poisoned, using stderr fallback");
                    drop(poisoned.into_inner());
                }
            }
        }
    };
}
// err lvl macro
macro_rules! log_err {
    ($logger:expr, $msg:expr) => {
        {
            match $logger.lock() {
                Ok(mut guard) => {
                    let _ = guard.error($msg);
                },
                Err(poisoned) => {
                    eprintln!("ERROR: {}", $msg);
                    eprintln!("WARNING: Logger mutex is poisoned, using stderr fallback");
                    drop(poisoned.into_inner());
                }
            }
        }
    };
}
// debug lvl macro
macro_rules! log_debug {
    ($logger:expr, $msg:expr) => {
        {
            match $logger.lock() {
                Ok(mut guard) => {
                    let _ = guard.debug($msg);
                },
                Err(poisoned) => {
                    eprintln!("DEBUG: {}", $msg);
                    eprintln!("WARNING: Logger mutex is poisoned, using stderr fallback");
                    drop(poisoned.into_inner());
                }
            }
        }
    };
}
// ------------------------------------------------------------------------------------------------

// Learn this shit later cuz Deepseek wrote this! -------------------------------------------------
type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    logger: Arc<Mutex<Logger>>,
}

impl ThreadPool {
    pub fn new(size: usize, logger: Arc<Mutex<Logger>>) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&logger)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            logger,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let job = Box::new(f);
            if let Err(e) = sender.send(job) {
                log_err!(self.logger, &format!("Failed to send job to thread pool: {}", e));
            }
        } else {
            log_err!(self.logger, "Thread pool is shutting down, job rejected");
        }
    }

    pub fn shutdown(&mut self) {
        log_info!(self.logger, "Shutting down thread pool...");

        // Закрываем канал
        drop(self.sender.take());

        // Ждем завершения всех потоков с таймаутом
        for (i, worker) in &mut self.workers.iter_mut().enumerate() {
            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(()) => {
                        log_debug!(self.logger, &format!("Worker {} stopped gracefully", i));
                    }
                    Err(e) => {
                        log_err!(self.logger, &format!("Worker {} failed to stop in time: {:?}", i, e));
                        // Можно добавить thread.terminate() если критично
                    }
                }
            }
        }
        log_info!(self.logger, "Thread pool shutdown complete");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.sender.is_some() {
            self.shutdown();
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, logger: Arc<Mutex<Logger>>) -> Worker {
        let thread = thread::spawn(move || {
            log_debug!(logger, &format!("Worker {} started", id));

            loop {
                let job = match receiver.lock() {
                    Ok(guard) => match guard.recv() {
                        Ok(job) => job,
                        Err(_) => {
                            // Канал закрыт - нормальное завершение
                            log_debug!(logger, &format!("Worker {} shutting down (channel closed)", id));
                            break;
                        }
                    },
                    Err(poisoned) => {
                        // Мьютекс poisoned - критическая ошибка
                        log_err!(logger, &format!("Worker {} - mutex poisoned, shutting down", id));
                        break;
                    }
                };

                // Выполняем задачу с обработкой паник
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    job();
                }));

                if let Err(panic) = result {
                    log_err!(logger, &format!("Worker {} - job panicked: {:?}", id, panic));
                    // Продолжаем работу, не падаем из-за одной задачи
                }
            }

            log_debug!(logger, &format!("Worker {} stopped", id));
        });

        Worker {
            id,
            thread: Some(thread)
        }
    }
}
// ------------------------------------------------------------------------------------------------

impl Server {
    // Main func
    pub fn run(mut self, max_threads: u8, mut logger: Logger) -> Result<(), Box<dyn std::error::Error>>{
        // self Arc
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        // logger Arc
        let logger = Arc::new(Mutex::new(logger));
        let logger_clone = Arc::clone(&logger);
        // create ThreadPool
        let pool = ThreadPool::new(max_threads as usize, Arc::clone(&logger));
        // listener
        let listener = self.listener();
        // Ctrl+C handler
        // ----------------------------------------------------------------------------------------
        ctrlc::set_handler(move || {
            log_info!(logger_clone, "Server shutting down...");
            running_clone.store(false, Ordering::SeqCst);
        }).map_err(|e| {
            log_err!(logger, &format!("Failed to set Ctrl+C handler: {}", e));
            Box::new(e) as Box<dyn std::error::Error>
        })?;
        log_info!(logger, "The server stop handler has been successfully registered\n\tYou can now stop the server by pressing Ctrl+C");
        //-----------------------------------------------------------------------------------------
        // Server start message
        log_info!(logger, &format!("Server set {} max_threads successfully", max_threads));
        // Set nonblocking mode
        self.unlock().map_err(|e| {
            log_err!(logger, &format!("Failed to listener nonblocking mode: {}", e));
            e
        })?;
        log_info!(logger, "Server set listener nonblocking mode successfully");
        // Main loop
        while running.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        // Server stop  message
        log_info!(logger, "Server stopped");
        // Return
        Ok(())
    }
}

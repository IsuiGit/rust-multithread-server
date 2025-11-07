pub use super::components::{Server, Monitor};
pub use super::utils::pre_build;

use crate::Logger;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::any::Any;

impl Server {
    pub fn run(self, max_threads: u8, mut logger: Logger) -> Result<(), Box<dyn std::error::Error>>{
        // self Arc
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        // logger Arc
        let logger = Arc::new(Mutex::new(logger));
        let logger_clone = Arc::clone(&logger);
        // Ctrl+C handler
        ctrlc::set_handler(move || {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("Server shutting down...");
            }
            running_clone.store(false, Ordering::SeqCst);
        }).map_err(|e| {
            if let Ok(mut logger) = logger.lock() {
                logger.error(&format!("Failed to set Ctrl+C handler: {}", e));
            }
            format!("Failed to set Ctrl+C handler: {}", e)
        })?;
        // Server start message
        if let Ok(mut logger) = logger.lock() {
            logger.info(&format!("Server started with {} max_threads", max_threads));
        }
        // Main loop
        while running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // Server stop  message
        if let Ok(mut logger) = logger.lock() {
            logger.info("Server stopped");
        }
        // Return
        Ok(())
    }
}

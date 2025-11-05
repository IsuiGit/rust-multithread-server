mod core;
mod logger;
mod interfaces;

use core::server::{Server, Monitor, pre_build};
use logger::log::Logger;
use interfaces::cli::parse;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let (name, host, port, log_file_path) = parse()?;
    let mut logger = Logger::new(&log_file_path)?;
    let (server, monitor) = pre_build(name, host, port)?;
    logger.info(&format!("\n{}", server));
    logger.info(&format!("\n{}", monitor));
    Ok(())
}

use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Utc;

pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

pub struct Logger {
    log_file: PathBuf
}

impl Logger {
    pub fn new(log_file: &str) -> io::Result<Self> {
        let final_log_file = PathBuf::from(log_file);
        if let Some(parent) = final_log_file.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let _file = OpenOptions::new().create(true).write(true).append(true).open(&final_log_file)?;
        Ok(Logger { log_file: final_log_file })
    }

    pub fn log_file_path(&self) -> &Path {
        &self.log_file
    }

    pub fn log_file_path_as_string(&self) -> String {
        self.log_file.to_string_lossy().into_owned()
    }

    pub fn info(&mut self, message: &str) -> io::Result<()> {
        self.write_with_level(LogLevel::Info, message)
    }

    pub fn warn(&mut self, message: &str) -> io::Result<()> {
        self.write_with_level(LogLevel::Warn, message)
    }

    pub fn error(&mut self, message: &str) -> io::Result<()> {
        self.write_with_level(LogLevel::Error, message)
    }

    pub fn debug(&mut self, message: &str) -> io::Result<()> {
        self.write_with_level(LogLevel::Debug, message)
    }

    fn write_with_level(&mut self, level: LogLevel, message: &str) -> io::Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let formatted_message = format!("[{}] [{}] {}", timestamp, level.as_str(), message);
        match level {
            LogLevel::Error => eprintln!("{}", formatted_message),
            _ => println!("{}", formatted_message),
        }
        let mut file = OpenOptions::new().create(true).write(true).append(true).open(&self.log_file)?;
        writeln!(file, "{}", formatted_message)?;
        file.flush()?;
        Ok(())
    }
}

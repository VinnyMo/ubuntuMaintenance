// Logger module for ubuntu-maintenance
// Provides timestamped logging to /var/log/ubuntu_maintenance.log

use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const PRIMARY_LOG: &str = "/var/log/ubuntu_maintenance.log";
const FALLBACK_LOG: &str = "/tmp/ubuntu_maintenance.log";

/// Log a message with timestamp
pub fn log_message(message: &str) {
    let timestamp = Local::now().format("%a %b %d %H:%M:%S %Y");
    let log_entry = format!("[{}] {}\n", timestamp, message);

    // Try primary log location first
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(PRIMARY_LOG)
    {
        let _ = file.write_all(log_entry.as_bytes());
    } else if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(FALLBACK_LOG)
    {
        let _ = file.write_all(log_entry.as_bytes());
    } else {
        eprintln!("WARNING: Cannot open log file");
    }
}

/// Log a command execution with its exit status
pub fn log_command(command: &str, success: bool) {
    if success {
        log_message(&format!("CMD: {}", command));
    } else {
        log_message(&format!("CMD FAILED: {}", command));
    }
}

/// Get the log file path being used
pub fn get_log_path() -> &'static str {
    if Path::new(PRIMARY_LOG).exists() {
        PRIMARY_LOG
    } else {
        FALLBACK_LOG
    }
}

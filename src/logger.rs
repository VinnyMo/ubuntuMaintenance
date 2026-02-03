// Logger module for ubuntu-maintenance
// Provides timestamped logging to /var/log/ubuntu_maintenance.log

use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const PRIMARY_LOG: &str = "/var/log/ubuntu_maintenance.log";
const FALLBACK_LOG: &str = "/tmp/ubuntu_maintenance.log";
const PRIMARY_VERBOSE_LOG: &str = "/var/log/ubuntu_maintenance_verbose.log";
const FALLBACK_VERBOSE_LOG: &str = "/tmp/ubuntu_maintenance_verbose.log";

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

/// Get the verbose log file path being used
pub fn get_verbose_log_path() -> &'static str {
    if Path::new(PRIMARY_VERBOSE_LOG).exists() || Path::new("/var/log").exists() {
        PRIMARY_VERBOSE_LOG
    } else {
        FALLBACK_VERBOSE_LOG
    }
}

/// Log verbatim command output with timestamp
pub fn log_verbose(command: &str, output: &str) {
    let timestamp = Local::now().format("%a %b %d %H:%M:%S %Y");
    let separator = "=".repeat(80);
    let log_entry = format!(
        "\n{}\n[{}] COMMAND: {}\n{}\n{}\n",
        separator, timestamp, command, separator, output
    );

    let verbose_path = get_verbose_log_path();

    // Try to write to verbose log
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(verbose_path)
    {
        let _ = file.write_all(log_entry.as_bytes());
    } else {
        eprintln!("WARNING: Cannot open verbose log file");
    }
}

/// Read the last N lines from the verbose log file (newest first)
pub fn read_verbose_log(max_lines: usize) -> anyhow::Result<Vec<String>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let verbose_path = get_verbose_log_path();
    let path = Path::new(verbose_path);

    if !path.exists() {
        return Ok(vec!["No verbose logs found.".to_string()]);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines()
        .filter_map(Result::ok)
        .collect();

    // Return last N lines, reversed (newest first)
    let start = if lines.len() > max_lines {
        lines.len() - max_lines
    } else {
        0
    };

    let mut result = lines[start..].to_vec();
    result.reverse();
    Ok(result)
}

// Utility functions module for ubuntu-maintenance
// Console output, system commands, and date/time formatting

use chrono::{Datelike, Local};
use colored::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::logger::log_command;

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Print message to user with default formatting (two newlines before, one after)
pub fn tell_user(message: &str) {
    println!("\n\n{}", message);
}

/// Print message with custom formatting
pub fn tell_user_custom(message: &str, leading_nl: usize, trailing_nl: usize) {
    for _ in 0..leading_nl {
        println!();
    }
    print!("{}", message);
    for _ in 0..trailing_nl {
        println!();
    }
    io::stdout().flush().unwrap();
}

/// Print message without any formatting
pub fn tell_user_no_format(message: &str) {
    print!("{}", message);
    io::stdout().flush().unwrap();
}

/// Execute a system command and display output
pub fn tell_system(command: &str) -> anyhow::Result<bool> {
    log_command(command, true);

    let output = if command.contains('|') || command.contains('>') {
        // Complex command - use shell
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?
    } else {
        // Simple command - parse and execute directly
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?
    };

    let success = output.status.success();

    if !success {
        eprintln!("WARNING: Command returned non-zero exit code: {}", command);
        log_command(command, false);
    }

    Ok(success)
}

/// Display formatted date and time information
pub fn custom_date_formatted() {
    let now = Local::now();

    let day = now.day();
    let suffix = get_number_suffix(day);

    let time_str = now.format("%I:%M%p");
    let date_str = now.format("%A, %B");
    let year = now.year();

    println!(
        "{} on {}, the {}{}, {}",
        time_str, date_str, day, suffix, year
    );

    // Display precise time with nanoseconds
    let timestamp = now.timestamp();
    let nanos = now.timestamp_subsec_nanos();
    println!(
        "At precisely {} seconds and {} nanoseconds",
        timestamp % 60,
        nanos
    );
}

/// Get the ordinal suffix for a number (st, nd, rd, th)
fn get_number_suffix(num: u32) -> &'static str {
    match (num % 10, num % 100) {
        (1, 11) => "th",
        (2, 12) => "th",
        (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

/// Check if running with sudo/root privileges
pub fn check_sudo() -> bool {
    nix::unistd::geteuid().is_root()
}

/// Display error message in red
pub fn error_message(msg: &str) {
    eprintln!("{}", msg.red().bold());
}

/// Display success message in green
pub fn success_message(msg: &str) {
    println!("{}", msg.green().bold());
}

/// Display warning message in yellow
pub fn warning_message(msg: &str) {
    println!("{}", msg.yellow().bold());
}

/// Get user input from stdin
pub fn get_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Get yes/no confirmation from user
pub fn confirm(prompt: &str) -> bool {
    tell_user_no_format(prompt);
    if let Ok(input) = get_input() {
        matches!(input.to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}

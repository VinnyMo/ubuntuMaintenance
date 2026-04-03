// Utility functions module for ubuntu-maintenance
// Console output, system commands, and date/time formatting

use chrono::{DateTime, Datelike, Local};
use colored::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::logger::{get_verbose_log_path, log_command, log_verbose, read_verbose_log};

const INFO_CACHE_TTL_SECONDS: i64 = 20;

struct CachedSystemInfo {
    fetched_at: DateTime<Local>,
    snapshot: SystemInfoSnapshot,
}

#[derive(Clone, Default)]
pub struct SystemInfoSnapshot {
    pub fetched_at: String,
    pub hostname: String,
    pub kernel: String,
    pub os: String,
    pub uptime: String,
    pub last_reboot: String,
    pub updates_count: i32,
    pub security_count: i32,
    pub disk: String,
    pub memory: String,
}

static SYSTEM_INFO_CACHE: OnceLock<Mutex<Option<CachedSystemInfo>>> = OnceLock::new();

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
#[allow(dead_code)]
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

pub fn section_heading(title: &str) {
    println!("{}", title.blue().bold());
}

pub fn show_banner(title: &str, subtitle: &str) {
    clear_screen();
    println!();
    println!("{}", "Ubuntu Maintenance".blue().bold());
    println!("{}", title.white().bold());
    if !subtitle.is_empty() {
        println!("{}", subtitle.dimmed());
    }
    println!("{}", "─".repeat(72).dimmed());
    println!();
}

pub fn wait_for_enter(prompt: &str) {
    println!();
    println!("{}", prompt.dimmed());
    let _ = get_input();
}

pub fn run_menu(title: &str, subtitle: &str, items: &[(&str, &str)]) -> Option<usize> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent},
        terminal::{disable_raw_mode, enable_raw_mode},
    };

    if items.is_empty() {
        return None;
    }

    let mut selected = 0;

    loop {
        show_banner(title, subtitle);

        for (index, (label, detail)) in items.iter().enumerate() {
            let marker = if index == selected { "›" } else { " " };
            let title_line = format!("{} {}", marker, label);
            if index == selected {
                println!("{}", title_line.green().bold());
                if !detail.is_empty() {
                    println!("    {}", detail.green());
                }
            } else {
                println!("{}", title_line);
                if !detail.is_empty() {
                    println!("    {}", detail.dimmed());
                }
            }
            println!();
        }

        println!(
            "{}",
            "Use ↑/↓ to move, Enter to choose, Esc or q to go back".dimmed()
        );

        if enable_raw_mode().is_err() {
            error_message("Failed to enable terminal raw mode");
            return None;
        }

        let result = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up => {
                        if selected == 0 {
                            selected = items.len() - 1;
                        } else {
                            selected -= 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected + 1 < items.len() {
                            selected += 1;
                        } else {
                            selected = 0;
                        }
                        break None;
                    }
                    KeyCode::Enter => break Some(selected),
                    KeyCode::Esc | KeyCode::Char('q') => break Some(items.len() - 1),
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        if let Some(choice) = result {
            return Some(choice);
        }
    }
}

/// Execute a system command and display output
#[allow(dead_code)]
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

/// Execute a system command silently with animated progress indicator
#[allow(dead_code)]
pub fn tell_system_with_progress(command: &str, message: &str) -> anyhow::Result<bool> {
    log_command(command, true);

    print!("{}", message);
    io::stdout().flush()?;

    // Start the command
    let mut child = if command.contains('|') || command.contains('>') {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
    } else {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
    };

    // Animate progress while command runs
    let mut dots = 0;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                println!(); // New line after progress
                let success = status.success();
                if success {
                    success_message("✓ Complete");
                } else {
                    error_message("✗ Failed");
                    log_command(command, false);
                }
                return Ok(success);
            }
            Ok(None) => {
                // Still running, show progress
                dots = (dots + 1) % 4;
                let indicator = ".".repeat(dots);
                print!("\r{}{:<3}", message, indicator);
                io::stdout().flush()?;
                thread::sleep(Duration::from_millis(500));
            }
            Err(e) => {
                println!();
                error_message(&format!("Error waiting for command: {}", e));
                return Err(e.into());
            }
        }
    }
}

/// Execute a system command with optional verbose output toggle
/// Press 'v' during execution to toggle between progress indicator and live output
pub fn tell_system_with_verbose(command: &str, message: &str) -> anyhow::Result<bool> {
    use crossterm::{
        event::{self, poll, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::sync::mpsc::{self, TryRecvError};
    use std::sync::{Arc, Mutex};

    log_command(command, true);

    // Start the command with piped output
    let mut child = if command.contains('|') || command.contains('>') {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
    } else {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
    };

    // Capture stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Create channels for output lines
    let (tx_out, rx_out) = mpsc::channel();
    let (tx_err, rx_err) = mpsc::channel();

    // Spawn thread to read stdout
    let tx_out_clone = tx_out.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = tx_out_clone.send(line);
            }
        }
    });

    // Spawn thread to read stderr
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = tx_err.send(line);
            }
        }
    });

    let mut verbose_mode = false;
    let mut dots = 0;
    let output_buffer = Arc::new(Mutex::new(String::new()));
    let output_buffer_clone = Arc::clone(&output_buffer);

    // Enable raw mode for key detection
    enable_raw_mode()?;

    // Initial progress display
    print!("{} {} ", message, "[v=verbose]".dimmed());
    io::stdout().flush()?;

    loop {
        // Read any available output
        loop {
            match rx_out.try_recv() {
                Ok(line) => {
                    if let Ok(mut buffer) = output_buffer.lock() {
                        buffer.push_str(&line);
                        buffer.push('\n');
                    }
                    if verbose_mode {
                        // Clear current line and print output
                        print!("\r{}\r", " ".repeat(80));
                        println!("  {}", line);
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        loop {
            match rx_err.try_recv() {
                Ok(line) => {
                    if let Ok(mut buffer) = output_buffer.lock() {
                        buffer.push_str(&line);
                        buffer.push('\n');
                    }
                    if verbose_mode {
                        print!("\r{}\r", " ".repeat(80));
                        println!("  {}", line.yellow());
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        // Check if process has finished
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished - read any remaining output
                thread::sleep(Duration::from_millis(100));
                loop {
                    match rx_out.try_recv() {
                        Ok(line) => {
                            if let Ok(mut buffer) = output_buffer_clone.lock() {
                                buffer.push_str(&line);
                                buffer.push('\n');
                            }
                            if verbose_mode {
                                print!("\r{}\r", " ".repeat(80));
                                println!("  {}", line);
                            }
                        }
                        Err(_) => break,
                    }
                }
                loop {
                    match rx_err.try_recv() {
                        Ok(line) => {
                            if let Ok(mut buffer) = output_buffer_clone.lock() {
                                buffer.push_str(&line);
                                buffer.push('\n');
                            }
                            if verbose_mode {
                                print!("\r{}\r", " ".repeat(80));
                                println!("  {}", line.yellow());
                            }
                        }
                        Err(_) => break,
                    }
                }

                // Disable raw mode
                let _ = disable_raw_mode();

                // Log verbatim output
                if let Ok(buffer) = output_buffer_clone.lock() {
                    log_verbose(command, &buffer);
                }

                // Clear line and show result
                print!("\r{}\r", " ".repeat(80));
                let success = status.success();
                if success {
                    println!("{} {}", message, "✓".green().bold());
                } else {
                    println!("{} {}", message, "✗".red().bold());
                    log_command(command, false);
                }
                return Ok(success);
            }
            Ok(None) => {
                // Still running - check for key presses
                if poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if let KeyCode::Char('v') | KeyCode::Char('V') = key.code {
                            verbose_mode = !verbose_mode;
                            // Clear line and show new mode
                            print!("\r{}\r", " ".repeat(80));
                            if verbose_mode {
                                println!(
                                    "{} {} {}",
                                    message,
                                    "[VERBOSE]".yellow().bold(),
                                    "press 'v' to hide".dimmed()
                                );
                            }
                        }
                    }
                }

                // Show progress indicator if not in verbose mode
                if !verbose_mode {
                    dots = (dots + 1) % 4;
                    let spinner = match dots {
                        0 => "⠋",
                        1 => "⠙",
                        2 => "⠹",
                        _ => "⠸",
                    };
                    print!("\r{} {} {}", message, "[v=verbose]".dimmed(), spinner);
                    io::stdout().flush()?;
                }

                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = disable_raw_mode();
                print!("\r{}\r", " ".repeat(80));
                error_message(&format!("{} - Error: {}", message, e));
                return Err(e.into());
            }
        }
    }
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

pub fn get_system_info_snapshot() -> SystemInfoSnapshot {
    let cache = SYSTEM_INFO_CACHE.get_or_init(|| Mutex::new(None));

    if let Ok(guard) = cache.lock() {
        if let Some(cached) = &*guard {
            if Local::now()
                .signed_duration_since(cached.fetched_at)
                .num_seconds()
                < INFO_CACHE_TTL_SECONDS
            {
                return cached.snapshot.clone();
            }
        }
    }

    let snapshot = SystemInfoSnapshot {
        fetched_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        hostname: run_command_capture("uname -n", "Unavailable"),
        kernel: run_command_capture("uname -r", "Unavailable"),
        os: run_command_capture("lsb_release -d 2>/dev/null | cut -f2", "Ubuntu"),
        uptime: run_command_capture("uptime -p", "Unavailable"),
        last_reboot: run_command_capture("who -b | awk '{print $3, $4}'", "Unavailable"),
        updates_count: run_command_capture(
            "apt list --upgradable 2>/dev/null | grep -v 'Listing...' | wc -l",
            "0",
        )
        .parse::<i32>()
        .unwrap_or(0),
        security_count: run_command_capture(
            "apt list --upgradable 2>/dev/null | grep -i security | wc -l",
            "0",
        )
        .parse::<i32>()
        .unwrap_or(0),
        disk: run_command_capture(
            "df -H / | tail -1 | awk '{print $5 \" used | \" $4 \" free | \" $2 \" total\"}'",
            "Unavailable",
        ),
        memory: run_command_capture(
            "free -h --si | awk 'NR==2 {print $3 \" used | \" $7 \" available | \" $2 \" total\"}'",
            "Unavailable",
        ),
    };

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CachedSystemInfo {
            fetched_at: Local::now(),
            snapshot: snapshot.clone(),
        });
    }

    snapshot
}

pub fn invalidate_system_info_cache() {
    if let Some(cache) = SYSTEM_INFO_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            *guard = None;
        }
    }
}

fn run_command_capture(command: &str, fallback: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Display countdown timer with cancel option
/// Returns true if countdown completed, false if cancelled
pub fn countdown_with_cancel(seconds: u32, message: &str) -> bool {
    use crossterm::{
        event::{self, poll, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::process::{Command as ProcessCommand, Stdio};

    println!("\n{}", message);
    println!();
    println!("{}", "Press 'c' to cancel reboot".yellow().bold());
    println!();

    if enable_raw_mode().is_err() {
        error_message("Failed to enable raw mode");
        return true; // Proceed with reboot if we can't enable raw mode
    }

    let mut remaining = seconds;

    while remaining > 0 {
        let minutes = remaining / 60;
        let secs = remaining % 60;

        print!("\r⏱  Time to reboot: {}:{:02} ", minutes, secs);
        io::stdout().flush().unwrap();

        // Check for key press (non-blocking)
        if poll(Duration::from_secs(1)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if let KeyCode::Char('c') | KeyCode::Char('C') = key.code {
                    let _ = disable_raw_mode();
                    println!("\n");

                    // Cancel the scheduled shutdown
                    let _ = ProcessCommand::new("sudo")
                        .arg("shutdown")
                        .arg("-c")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .output();

                    return false;
                }
            }
        }

        remaining -= 1;
    }

    let _ = disable_raw_mode();
    println!("\n");
    success_message("Reboot proceeding...");
    true
}

/// Display verbose log viewer with pagination
pub fn view_verbose_logs() {
    use crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };

    show_banner(
        "Logs",
        "Browse the detailed command log captured during update runs.",
    );
    println!("Log file: {}\n", get_verbose_log_path().yellow());

    match read_verbose_log(500) {
        Ok(lines) => {
            if lines.is_empty() || (lines.len() == 1 && lines[0].contains("No verbose logs")) {
                warning_message("No verbose logs found yet.");
                println!("\nVerbose logs will be created when you run updates.");
                println!("These logs contain the full command output for troubleshooting.");
            } else {
                let page_size = 18;
                let mut offset = 0;
                let total_lines = lines.len();

                loop {
                    show_banner(
                        "Logs",
                        "Browse the detailed command log captured during update runs.",
                    );
                    println!("Log file: {}", get_verbose_log_path().yellow());
                    println!(
                        "Showing lines {}-{} of {}",
                        offset + 1,
                        (offset + page_size).min(total_lines),
                        total_lines
                    );
                    println!("{}", "Newer entries are at the top.".dimmed());
                    println!("{}", "─".repeat(72).dimmed());

                    // Display current page
                    let end = (offset + page_size).min(total_lines);
                    for line in &lines[offset..end] {
                        println!("{}", line);
                    }

                    println!("{}", "─".repeat(72).dimmed());
                    println!();
                    println!(
                        "{}",
                        "↑/↓ scroll one line, PgUp/PgDn scroll a page, Home/End jump, q exits"
                            .dimmed()
                    );

                    // Enable raw mode for key detection
                    if enable_raw_mode().is_err() {
                        break;
                    }

                    // Wait for key press
                    let should_quit = if let Ok(Event::Key(key)) = event::read() {
                        match key.code {
                            KeyCode::Up => {
                                if offset > 0 {
                                    offset = offset.saturating_sub(1);
                                }
                                false
                            }
                            KeyCode::Down => {
                                if offset + page_size < total_lines {
                                    offset += 1;
                                }
                                false
                            }
                            KeyCode::PageUp => {
                                offset = offset.saturating_sub(page_size);
                                false
                            }
                            KeyCode::PageDown => {
                                if offset + page_size < total_lines {
                                    offset = (offset + page_size).min(total_lines - page_size);
                                }
                                false
                            }
                            KeyCode::Home => {
                                offset = 0;
                                false
                            }
                            KeyCode::End => {
                                offset = total_lines.saturating_sub(page_size);
                                false
                            }
                            KeyCode::Char('q') | KeyCode::Esc => true,
                            _ => false,
                        }
                    } else {
                        true
                    };

                    let _ = disable_raw_mode();

                    if should_quit {
                        break;
                    }
                }
            }
        }
        Err(e) => {
            error_message(&format!("Error reading log file: {}", e));
        }
    }

    wait_for_enter("Press Enter to return to the menu...");
}

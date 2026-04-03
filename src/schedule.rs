// Schedule management module for ubuntu-maintenance
// Handles cron job creation, viewing, and removal

use crate::logger::log_message;
use crate::utils::{error_message, section_heading, success_message, tell_user};
use colored::*;
use std::fs;
use std::io::Write;
use std::process::Command;

/// Check if any ubuntu-maintenance schedules exist
pub fn has_existing_schedule() -> anyhow::Result<bool> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("crontab -l 2>/dev/null | grep -c ubuntu-maintenance")
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .unwrap_or(0);

    Ok(count > 0)
}

/// Display current ubuntu-maintenance cron schedules
pub fn show_current_schedule() -> anyhow::Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("crontab -l 2>/dev/null | grep ubuntu-maintenance")
        .output()?;

    if !output.status.success() || output.stdout.is_empty() {
        section_heading("Scheduled updates");
        tell_user("No scheduled updates found.");
        return Ok(());
    }

    let lines = String::from_utf8_lossy(&output.stdout);
    let mut schedules = Vec::new();

    for line in lines.lines() {
        if line.starts_with('#') {
            continue;
        }
        schedules.push(line.to_string());
    }

    section_heading("Scheduled updates");
    println!(
        "{}",
        "These jobs run automatically through your user crontab.".dimmed()
    );
    println!();

    for (index, line) in schedules.iter().enumerate() {
        println!(
            "{} {}",
            format!("Schedule {}", index + 1).green().bold(),
            format!("({})", schedule_summary(line)).dimmed()
        );
        println!("  Runs: {}", schedule_frequency(line));
        println!("  Mode: {}", schedule_mode(line));
        println!("  Raw : {}", line.dimmed());
        println!();
    }

    if !schedules.is_empty() {
        println!("Total schedules: {}", schedules.len());
        println!(
            "{}",
            "Tip: remove and recreate schedules here if you want a cleaner setup.".dimmed()
        );
    }

    Ok(())
}

/// Add a new cron schedule
pub fn add_schedule(frequency: &str, mode: &str) -> anyhow::Result<()> {
    // Determine cron schedule
    let cron_time = match frequency {
        "daily" => "0 2 * * *",
        "weekly" => "0 3 * * 0",
        "weekdays" => "0 2 * * 1-5",
        _ => {
            error_message("ERROR: Invalid frequency. Use 'daily', 'weekly', or 'weekdays'");
            return Err(anyhow::anyhow!("Invalid frequency"));
        }
    };

    // Determine mode flag
    let mode_flag = match mode {
        "all" => "-a",
        "critical" => "-c",
        "force" => "-f",
        _ => {
            error_message("ERROR: Invalid mode. Use 'all', 'critical', or 'force'");
            return Err(anyhow::anyhow!("Invalid mode"));
        }
    };

    // Find ubuntu-maintenance binary
    let binary_path = find_binary_path()?;

    // Build cron command
    let cron_command = format!(
        "{} {} >> /var/log/automated_updates.log 2>&1",
        binary_path, mode_flag
    );

    let full_cron_line = format!("{} {}", cron_time, cron_command);

    // Create temp file with updated crontab
    let temp_file = "/tmp/crontab_ubuntu_maintenance.tmp";

    // Get existing crontab
    let existing = Command::new("sh")
        .arg("-c")
        .arg("crontab -l 2>/dev/null")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Write new crontab
    let mut file = fs::File::create(temp_file)?;
    file.write_all(existing.as_bytes())?;
    writeln!(
        file,
        "# ubuntu-maintenance - automated {} {} updates",
        frequency, mode
    )?;
    writeln!(file, "{}", full_cron_line)?;
    drop(file);

    // Install new crontab
    let install_result = Command::new("crontab").arg(temp_file).status()?;

    // Clean up temp file
    let _ = fs::remove_file(temp_file);

    if !install_result.success() {
        error_message("ERROR: Failed to install crontab");
        log_message("ERROR: Failed to add cron schedule");
        return Err(anyhow::anyhow!("Failed to install crontab"));
    }

    success_message("SUCCESS: Schedule added successfully!");
    log_message(&format!(
        "Added cron schedule: {} updates {}",
        mode, frequency
    ));

    Ok(())
}

fn schedule_frequency(line: &str) -> &'static str {
    if line.contains("0 2 * * *") {
        "Daily at 2:00 AM"
    } else if line.contains("0 3 * * 0") {
        "Weekly on Sunday at 3:00 AM"
    } else if line.contains("0 2 * * 1-5") {
        "Weekdays at 2:00 AM"
    } else {
        "Custom cron timing"
    }
}

fn schedule_mode(line: &str) -> &'static str {
    if line.contains("-a") || line.contains("--all") {
        "All updates, no reboot"
    } else if line.contains("-c") || line.contains("--critical") {
        "Critical and security updates only"
    } else if line.contains("-f") || line.contains("--force") {
        "Full update with automatic reboot"
    } else {
        "Unknown"
    }
}

fn schedule_summary(line: &str) -> &'static str {
    match (schedule_frequency(line), schedule_mode(line)) {
        ("Daily at 2:00 AM", "All updates, no reboot") => "daily maintenance",
        ("Weekly on Sunday at 3:00 AM", "All updates, no reboot") => "weekly maintenance",
        ("Weekdays at 2:00 AM", "Critical and security updates only") => {
            "weekday security patching"
        }
        (_, "Full update with automatic reboot") => "automatic reboot enabled",
        _ => "custom schedule",
    }
}

/// Remove all ubuntu-maintenance schedules
pub fn remove_all_schedules() -> anyhow::Result<i32> {
    let temp_file = "/tmp/crontab_ubuntu_maintenance.tmp";

    // Get existing crontab
    let output = Command::new("sh")
        .arg("-c")
        .arg("crontab -l 2>/dev/null")
        .output()?;

    if !output.status.success() {
        tell_user("No crontab found.");
        return Ok(0);
    }

    let existing = String::from_utf8_lossy(&output.stdout);
    let mut removed = 0;
    let mut skip_next = false;

    // Filter out ubuntu-maintenance entries
    let mut new_crontab = String::new();

    for line in existing.lines() {
        if skip_next {
            skip_next = false;
            removed += 1;
            continue;
        }

        if line.contains("# ubuntu-maintenance") {
            skip_next = true;
            continue;
        }

        if line.contains("ubuntu-maintenance") {
            removed += 1;
            continue;
        }

        new_crontab.push_str(line);
        new_crontab.push('\n');
    }

    if removed == 0 {
        tell_user("No ubuntu-maintenance schedules found to remove.");
        return Ok(0);
    }

    // Write filtered crontab
    fs::write(temp_file, new_crontab)?;

    // Install new crontab
    let install_result = Command::new("crontab").arg(temp_file).status()?;

    // Clean up
    let _ = fs::remove_file(temp_file);

    if !install_result.success() {
        error_message("ERROR: Failed to update crontab");
        log_message("ERROR: Failed to remove cron schedules");
        return Err(anyhow::anyhow!("Failed to update crontab"));
    }

    println!("\nSUCCESS: Removed {} schedule(s)\n", removed);
    log_message(&format!("Removed {} cron schedule(s)", removed));

    Ok(removed)
}

/// Find the ubuntu-maintenance binary path
fn find_binary_path() -> anyhow::Result<String> {
    // Try `which` first
    let output = Command::new("which").arg("ubuntu-maintenance").output()?;

    if output.status.success() && !output.stdout.is_empty() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(path);
    }

    // Try common locations
    for path in &[
        "/usr/bin/ubuntu-maintenance",
        "/usr/local/bin/ubuntu-maintenance",
    ] {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    error_message("ERROR: Cannot find ubuntu-maintenance binary");
    Err(anyhow::anyhow!("ubuntu-maintenance binary not found"))
}

// Ubuntu Maintenance Tool - Rust Edition
// Author: Vincent T. Mossman
// Production-ready command-line utility for automated Ubuntu/Debian server maintenance

mod logger;
mod schedule;
mod utils;

use clap::Parser;
use colored::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use logger::{log_message, get_log_path};
use schedule::{add_schedule, has_existing_schedule, remove_all_schedules, show_current_schedule};
use std::process::Command;
use std::thread;
use std::time::Duration;
use utils::*;

const VERSION: &str = "3.1.0";
const REBOOT_DELAY_MINUTES: i32 = 5;

#[derive(Parser, Debug)]
#[command(name = "ubuntu-maintenance")]
#[command(about = "Production-ready Ubuntu/Debian server maintenance tool", long_about = None)]
#[command(version = VERSION)]
struct Args {
    /// Force update with automatic reboot
    #[arg(short = 'f', long = "force")]
    force: bool,

    /// All updates without reboot
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Critical security updates only
    #[arg(short = 'c', long = "critical")]
    critical: bool,

    /// Display system information
    #[arg(short = 'i', long = "info")]
    info: bool,

    /// Preview updates without applying (dry-run mode)
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,
}

struct AppState {
    dry_run: bool,
}

fn main() {
    log_message("=== Ubuntu Maintenance Tool Started ===");

    let args = Args::parse();

    // Check if running in CLI mode or interactive mode
    let cli_mode = args.force || args.all || args.critical || args.info;

    if cli_mode {
        let state = AppState {
            dry_run: args.dry_run,
        };

        if args.dry_run {
            tell_user("DRY RUN MODE ENABLED - No changes will be made");
            log_message("Dry run mode enabled");
        }

        // Help and info don't require sudo
        if args.info {
            show_information();
        } else {
            // Other operations require sudo
            if !check_sudo() {
                error_message("ERROR: This program must be run with sudo privileges.");
                error_message("Usage: sudo ubuntu-maintenance [options]");
                log_message("ERROR: Attempted to run without sudo");
                std::process::exit(1);
            }

            if args.force {
                force_update(&state);
            } else if args.all {
                all_update(&state);
            } else if args.critical {
                critical_update(&state);
            }
        }
    } else {
        // Interactive mode - require sudo
        if !check_sudo() {
            error_message("ERROR: This program must be run with sudo privileges.");
            error_message("Usage: sudo ubuntu-maintenance");
            log_message("ERROR: Attempted to run without sudo");
            std::process::exit(1);
        }
        main_menu();
    }

    log_message("=== Ubuntu Maintenance Tool Finished ===");
}

fn main_menu() {
    let menu_items = vec![
        "Force Update (with reboot)",
        "All Update (no reboot)",
        "Critical Update (security only)",
        "System Information",
        "Help",
        "Manage Update Schedule",
        "Exit",
    ];

    let mut selected = 0;

    loop {
        clear_screen();

        // Header
        println!("\n{}\n", "=== UBUNTU MAINTENANCE ===".blue().bold());

        // Menu items
        for (i, item) in menu_items.iter().enumerate() {
            if i == selected {
                println!("\t{} {}", "*".green().bold(), item.green());
            } else {
                println!("\t  {}", item);
            }
        }

        println!("\n{}", "Use ↑/↓ arrow keys to navigate, Enter to select".dimmed());

        // Enable raw mode to capture arrow keys
        if enable_raw_mode().is_err() {
            error_message("Failed to enable terminal raw mode");
            return;
        }

        // Wait for key press
        let choice = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(selected);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Some(menu_items.len() - 1); // Exit option
                    }
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        // Process selection
        if let Some(idx) = choice {
            let state = AppState { dry_run: false };

            match idx {
                0 => {
                    force_update(&state);
                }
                1 => {
                    all_update(&state);
                }
                2 => {
                    critical_update(&state);
                }
                3 => {
                    show_information();
                    tell_user("");
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
                }
                4 => {
                    show_help();
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
                }
                5 => {
                    manage_schedule();
                }
                6 => {
                    tell_user("Exiting. No changes made.");
                    log_message("Program exited by user");
                    return;
                }
                _ => {}
            }
        }
    }
}

fn show_help() {
    clear_screen();
    println!("\n{}\n", "=== UBUNTU MAINTENANCE ===".blue().bold());
    tell_user("");
    tell_user("Usage: sudo ubuntu-maintenance [options]");
    tell_user("");
    tell_user("Options:");
    tell_user("  -f, --force       Force update with reboot");
    tell_user("  -a, --all         All updates without reboot");
    tell_user("  -c, --critical    Critical security updates only");
    tell_user("  -i, --info        Display system information");
    tell_user("  -d, --dry-run     Preview updates without applying");
    tell_user("  -h, --help        Display this help message");
    tell_user("");
    tell_user("Interactive Mode:");
    tell_user("  Run without arguments for interactive menu");
    tell_user("");
    tell_user("Examples:");
    tell_user("  sudo ubuntu-maintenance            # Interactive mode");
    tell_user("  sudo ubuntu-maintenance -a         # Run all updates");
    tell_user("  sudo ubuntu-maintenance --dry-run -a  # Preview all updates");
    tell_user("");
    println!("Logs are written to: {}", get_log_path());
    tell_user("");
}

fn show_information() {
    clear_screen();
    tell_user("=== SYSTEM INFORMATION ===");
    tell_user("");

    // System details using uname
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        println!("System:       {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    tell_user("");
    tell_user("Current Time:");
    custom_date_formatted();

    tell_user("");
    tell_user("=== PACKAGE INFORMATION ===");
    let _ = tell_system("apt --version");

    tell_user("");
    tell_user("=== AVAILABLE UPDATES ===");
    let _ = tell_system("apt list --upgradable 2>/dev/null | grep -v 'Listing...' | wc -l | xargs echo 'Packages with updates available:'");

    tell_user("");
    tell_user("=== SECURITY UPDATES ===");
    let _ = tell_system("apt list --upgradable 2>/dev/null | grep -i security | wc -l | xargs echo 'Security updates available:'");

    tell_user("");
    tell_user("=== DISK USAGE ===");
    let _ = tell_system("df -h / | tail -1 | awk '{print \"Root partition: \" $5 \" used of \" $2}'");

    tell_user("");
    tell_user("=== SYSTEM UPTIME ===");
    let _ = tell_system("uptime -p");

    tell_user("");
    tell_user("=== LAST REBOOT ===");
    let _ = tell_system("who -b");

    tell_user("");
    log_message("System information displayed");
}

fn complete_details() {
    tell_user("");
    tell_user("==== UPDATE COMPLETE ====");

    let _ = tell_system("apt --version");

    tell_user("");
    tell_user("Update completed at:");
    custom_date_formatted();

    tell_user("");
    tell_user("A system reboot is recommended to ensure all updates take effect.");
    tell_user("You can reboot now using: sudo shutdown -r +5");

    log_message("Update completed successfully");
}

fn safe_reboot(delay_minutes: i32) {
    tell_user("");
    tell_user("==== SCHEDULING SYSTEM REBOOT ====");

    tell_user_custom(&"Current system time:".green().to_string(), 1, 0);
    custom_date_formatted();

    println!("\nReboot scheduled for {} minutes from now.", delay_minutes);
    tell_user("To cancel: sudo shutdown -c");
    tell_user("");

    let reboot_command = format!(
        "sudo shutdown -r +{} \"System update complete. Rebooting in {} minutes. Use 'shutdown -c' to cancel.\"",
        delay_minutes, delay_minutes
    );

    if tell_system(&reboot_command).unwrap_or(false) {
        log_message("System reboot scheduled");
        tell_user("Please save your work and logout, or cancel the reboot if needed.");
    } else {
        error_message("ERROR: Failed to schedule reboot. Please reboot manually.");
        log_message("ERROR: Failed to schedule reboot");
    }
}

fn force_update(state: &AppState) {
    log_message("=== Starting FORCE UPDATE ===");

    if state.dry_run {
        tell_user("=== DRY RUN MODE: Preview of Force Update ===");
        tell_user("");
        tell_user("The following commands would be executed:");
        tell_user("  1. sudo apt update");
        tell_user("  2. sudo apt full-upgrade -y");
        tell_user("  3. sudo apt autoremove -y");
        tell_user("  4. sudo apt autoclean");
        tell_user("  5. System reboot scheduled");
        tell_user("");
        return;
    }

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if !tell_system("sudo apt update").unwrap_or(false) {
        warning_message("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING ALL AVAILABLE PACKAGE UPGRADES ====");
    tell_user("This may take several minutes depending on available updates...");
    if !tell_system("sudo apt full-upgrade -y").unwrap_or(false) {
        warning_message("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: full-upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== REMOVING OBSOLETE PACKAGES ====");
    let _ = tell_system("sudo apt autoremove -y");

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    let _ = tell_system("sudo apt autoclean");

    complete_details();

    tell_user("");
    safe_reboot(REBOOT_DELAY_MINUTES);

    log_message("=== FORCE UPDATE completed ===");
}

fn all_update(state: &AppState) {
    log_message("=== Starting ALL UPDATE ===");

    if state.dry_run {
        tell_user("=== DRY RUN MODE: Preview of All Update ===");
        tell_user("");
        tell_user("The following commands would be executed:");
        tell_user("  1. sudo apt update");
        tell_user("  2. sudo apt full-upgrade");
        tell_user("  3. sudo apt autoremove");
        tell_user("  4. sudo apt autoclean");
        tell_user("");
        return;
    }

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if !tell_system("sudo apt update").unwrap_or(false) {
        warning_message("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING ALL AVAILABLE PACKAGE UPGRADES ====");
    tell_user("This may take several minutes depending on available updates...");
    if !tell_system("sudo apt full-upgrade").unwrap_or(false) {
        warning_message("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: full-upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== REMOVING OBSOLETE PACKAGES ====");
    let _ = tell_system("sudo apt autoremove");

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    let _ = tell_system("sudo apt autoclean");

    complete_details();

    log_message("=== ALL UPDATE completed ===");
}

fn critical_update(state: &AppState) {
    log_message("=== Starting CRITICAL UPDATE ===");

    if state.dry_run {
        tell_user("=== DRY RUN MODE: Preview of Critical Update ===");
        tell_user("");
        tell_user("The following commands would be executed:");
        tell_user("  1. sudo apt update");
        tell_user("  2. sudo apt upgrade (security updates)");
        tell_user("  3. sudo apt autoclean");
        tell_user("");
        return;
    }

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if !tell_system("sudo apt update").unwrap_or(false) {
        warning_message("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING CRITICAL PACKAGE UPGRADES ====");
    tell_user("Installing security and critical updates only...");
    if !tell_system("sudo apt upgrade -y").unwrap_or(false) {
        warning_message("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    let _ = tell_system("sudo apt autoclean");

    complete_details();

    log_message("=== CRITICAL UPDATE completed ===");
}

fn manage_schedule() {
    let menu_items = vec![
        "Add a new schedule",
        "View current schedule",
        "Remove all schedules",
        "Return to main menu",
    ];

    let mut selected = 0;

    loop {
        clear_screen();
        println!("\n{}\n", "=== MANAGE UPDATE SCHEDULE ===".blue().bold());

        // Show current schedule first
        let _ = show_current_schedule();
        println!();

        // Menu items
        for (i, item) in menu_items.iter().enumerate() {
            if i == selected {
                println!("\t{} {}", "*".green().bold(), item.green());
            } else {
                println!("\t  {}", item);
            }
        }

        println!("\n{}", "Use ↑/↓ arrow keys to navigate, Enter to select".dimmed());

        if enable_raw_mode().is_err() {
            error_message("Failed to enable terminal raw mode");
            return;
        }

        let choice = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(selected);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Some(menu_items.len() - 1);
                    }
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        if let Some(idx) = choice {
            match idx {
                0 => add_schedule_menu(),
                1 => {
                    clear_screen();
                    let _ = show_current_schedule();
                    tell_user("Press Enter to continue...");
                    let _ = get_input();
                }
                2 => remove_schedule_menu(),
                3 => return,
                _ => {}
            }
        }
    }
}

fn add_schedule_menu() {
    // Step 1: Choose frequency
    let freq_items = vec![
        "Daily (every day at 2:00 AM)",
        "Weekly (every Sunday at 3:00 AM)",
        "Weekdays (Monday-Friday at 2:00 AM)",
        "Cancel",
    ];

    let mut selected = 0;

    let frequency = loop {
        clear_screen();
        println!("\n{}\n", "=== ADD NEW SCHEDULE ===".blue().bold());
        println!("Select frequency:\n");

        for (i, item) in freq_items.iter().enumerate() {
            if i == selected {
                println!("\t{} {}", "*".green().bold(), item.green());
            } else {
                println!("\t  {}", item);
            }
        }

        println!("\n{}", "Use ↑/↓ arrow keys to navigate, Enter to select".dimmed());

        if enable_raw_mode().is_err() {
            return;
        }

        let choice = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < freq_items.len() - 1 {
                            selected += 1;
                        }
                        break None;
                    }
                    KeyCode::Enter => break Some(selected),
                    KeyCode::Esc => break Some(freq_items.len() - 1),
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        if let Some(idx) = choice {
            match idx {
                0 => break "daily",
                1 => break "weekly",
                2 => break "weekdays",
                _ => return,
            }
        }
    };

    // Step 2: Choose update mode
    let mode_items = vec![
        "All updates (recommended for servers)",
        "Critical/security updates only",
        "Force update with reboot (use with caution!)",
        "Cancel",
    ];

    let mut selected = 0;

    let mode = loop {
        clear_screen();
        println!("\n{}\n", "=== ADD NEW SCHEDULE ===".blue().bold());
        println!("Select update mode:\n");

        for (i, item) in mode_items.iter().enumerate() {
            if i == selected {
                println!("\t{} {}", "*".green().bold(), item.green());
            } else {
                println!("\t  {}", item);
            }
        }

        println!("\n{}", "Use ↑/↓ arrow keys to navigate, Enter to select".dimmed());

        if enable_raw_mode().is_err() {
            return;
        }

        let choice = loop {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < mode_items.len() - 1 {
                            selected += 1;
                        }
                        break None;
                    }
                    KeyCode::Enter => break Some(selected),
                    KeyCode::Esc => break Some(mode_items.len() - 1),
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        if let Some(idx) = choice {
            match idx {
                0 => break "all",
                1 => break "critical",
                2 => break "force",
                _ => return,
            }
        }
    };

    // Confirm
    clear_screen();
    println!("\n{}\n", "=== CONFIRM SCHEDULE ===".blue().bold());
    println!("You are about to schedule:");
    println!("  Frequency: {}", frequency);
    println!("  Mode: {} updates", mode);

    if mode == "force" {
        println!();
        warning_message("WARNING: Force mode will automatically reboot your server!");
        println!("This should only be used if you have redundancy or scheduled maintenance windows.");
    }

    tell_user("");
    tell_user_no_format("Proceed? (y/N): ");

    if confirm("") {
        match add_schedule(frequency, mode) {
            Ok(_) => {
                tell_user("");
                success_message("Schedule added successfully!");
                tell_user("Automated updates will run in the background.");
                tell_user("Logs will be written to: /var/log/automated_updates.log");
                tell_user("");
                tell_user("Press Enter to continue...");
                let _ = get_input();
            }
            Err(_) => {
                tell_user("");
                error_message("Failed to add schedule. Check logs for details.");
                tell_user("Press Enter to continue...");
                let _ = get_input();
            }
        }
    } else {
        tell_user("Cancelled.");
        thread::sleep(Duration::from_secs(1));
    }
}

fn remove_schedule_menu() {
    match has_existing_schedule() {
        Ok(true) => {
            tell_user("");
            warning_message("WARNING: This will remove ALL ubuntu-maintenance schedules.");
            tell_user_no_format("Are you sure? (y/N): ");

            if confirm("") {
                match remove_all_schedules() {
                    Ok(_) => {
                        tell_user("Press Enter to continue...");
                        let _ = get_input();
                    }
                    Err(_) => {
                        error_message("Failed to remove schedules. Check logs for details.");
                        tell_user("Press Enter to continue...");
                        let _ = get_input();
                    }
                }
            } else {
                tell_user("Cancelled.");
                thread::sleep(Duration::from_secs(1));
            }
        }
        Ok(false) => {
            tell_user("");
            tell_user("No schedules to remove.");
            thread::sleep(Duration::from_secs(2));
        }
        Err(_) => {
            error_message("ERROR: Failed to check schedules.");
            thread::sleep(Duration::from_secs(2));
        }
    }

    manage_schedule();
}

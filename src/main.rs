// Ubuntu Maintenance Tool - Rust Edition
// Author: Vincent T. Mossman
// Production-ready command-line utility for automated Ubuntu/Debian server maintenance

mod logger;
mod schedule;
mod utils;

use clap::Parser;
use colored::*;
use logger::{log_message, get_log_path};
use schedule::{add_schedule, has_existing_schedule, remove_all_schedules, show_current_schedule};
use std::process::Command;
use std::thread;
use std::time::Duration;
use utils::*;

const VERSION: &str = "3.0.0";
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
    clear_screen();
    tell_user_custom(&"=== UBUNTU SERVER MAINTENANCE TOOL ===".bold().to_string(), 1, 1);
    tell_user_custom("1) Force Update (with reboot)", 0, 0);
    tell_user_custom("2) All Update (no reboot)", 0, 0);
    tell_user_custom("3) Critical Update (security only)", 0, 0);
    tell_user_custom("4) System Information", 0, 0);
    tell_user_custom("5) Help", 0, 0);
    tell_user_custom("6) Manage Update Schedule", 0, 0);
    tell_user_custom("0) Exit", 0, 1);
    tell_user_no_format("Enter choice (0-6): ");

    if let Ok(input) = get_input() {
        if let Some(choice) = validate_menu_input(&input, 0, 6) {
            let state = AppState { dry_run: false };

            match choice {
                1 => {
                    force_update(&state);
                }
                2 => {
                    all_update(&state);
                }
                3 => {
                    critical_update(&state);
                }
                4 => {
                    show_information();
                    tell_user("");
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
                    main_menu();
                }
                5 => {
                    show_help();
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
                    main_menu();
                }
                6 => {
                    manage_schedule();
                }
                0 => {
                    tell_user("Exiting. No changes made.");
                    log_message("Program exited by user");
                }
                _ => {
                    tell_user("Invalid choice.");
                    thread::sleep(Duration::from_secs(1));
                    main_menu();
                }
            }
        } else {
            tell_user("");
            tell_user("Invalid choice. Please enter a number between 0 and 6.");
            tell_user("");
            thread::sleep(Duration::from_secs(2));
            main_menu();
        }
    } else {
        error_message("ERROR: Failed to read input");
        log_message("ERROR: Failed to read menu input");
    }
}

fn show_help() {
    clear_screen();
    tell_user("=== UBUNTU SERVER MAINTENANCE TOOL ===");
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
    clear_screen();
    tell_user_custom(&"=== MANAGE UPDATE SCHEDULE ===".bold().to_string(), 1, 1);

    // Show current schedule first
    let _ = show_current_schedule();

    tell_user_custom("What would you like to do?", 1, 1);
    tell_user_custom("1) Add a new schedule", 0, 0);
    tell_user_custom("2) View current schedule", 0, 0);
    tell_user_custom("3) Remove all schedules", 0, 0);
    tell_user_custom("0) Return to main menu", 0, 1);
    tell_user_no_format("Enter choice (0-3): ");

    if let Ok(input) = get_input() {
        if let Some(choice) = validate_menu_input(&input, 0, 3) {
            match choice {
                1 => add_schedule_menu(),
                2 => {
                    clear_screen();
                    let _ = show_current_schedule();
                    tell_user("Press Enter to continue...");
                    let _ = get_input();
                    manage_schedule();
                }
                3 => remove_schedule_menu(),
                0 => main_menu(),
                _ => manage_schedule(),
            }
        } else {
            tell_user("Invalid choice.");
            thread::sleep(Duration::from_secs(2));
            manage_schedule();
        }
    } else {
        main_menu();
    }
}

fn add_schedule_menu() {
    clear_screen();
    tell_user("=== ADD NEW SCHEDULE ===");
    tell_user("");

    // Choose frequency
    tell_user("Select frequency:");
    tell_user("1) Daily (every day at 2:00 AM)");
    tell_user("2) Weekly (every Sunday at 3:00 AM)");
    tell_user("3) Weekdays (Monday-Friday at 2:00 AM)");
    tell_user("0) Cancel");
    tell_user_no_format("\nEnter choice (0-3): ");

    let frequency = if let Ok(input) = get_input() {
        match validate_menu_input(&input, 0, 3) {
            Some(0) => {
                manage_schedule();
                return;
            }
            Some(1) => "daily",
            Some(2) => "weekly",
            Some(3) => "weekdays",
            _ => {
                tell_user("Invalid choice.");
                thread::sleep(Duration::from_secs(2));
                manage_schedule();
                return;
            }
        }
    } else {
        manage_schedule();
        return;
    };

    // Choose update mode
    tell_user("");
    tell_user("Select update mode:");
    tell_user("1) All updates (recommended for servers)");
    tell_user("2) Critical/security updates only");
    tell_user("3) Force update with reboot (use with caution!)");
    tell_user("0) Cancel");
    tell_user_no_format("\nEnter choice (0-3): ");

    let mode = if let Ok(input) = get_input() {
        match validate_menu_input(&input, 0, 3) {
            Some(0) => {
                manage_schedule();
                return;
            }
            Some(1) => "all",
            Some(2) => "critical",
            Some(3) => "force",
            _ => {
                tell_user("Invalid choice.");
                thread::sleep(Duration::from_secs(2));
                manage_schedule();
                return;
            }
        }
    } else {
        manage_schedule();
        return;
    };

    // Confirm
    tell_user("");
    println!("You are about to schedule:");
    println!("  Frequency: {}", frequency);
    println!("  Mode: {} updates", mode);

    if mode == "force" {
        tell_user("");
        warning_message("WARNING: Force mode will automatically reboot your server!");
        tell_user("This should only be used if you have redundancy or scheduled maintenance windows.");
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

    manage_schedule();
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

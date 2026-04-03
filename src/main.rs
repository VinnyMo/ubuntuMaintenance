// Ubuntu Maintenance Tool - Rust Edition
// Author: Vincent T. Mossman
// Production-ready command-line utility for automated Ubuntu/Debian server maintenance

mod logger;
mod schedule;
mod utils;

use clap::Parser;
use colored::*;
use logger::{get_log_path, log_message};
use schedule::{add_schedule, has_existing_schedule, remove_all_schedules, show_current_schedule};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use utils::*;

const VERSION: &str = "3.1.11";
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
    let menu_items = [
        (
            "Run Updates",
            "Choose between all updates, security-only updates, or a rebooting maintenance run.",
        ),
        (
            "Manage Schedule",
            "Review scheduled jobs, add a new automation rule, or remove old cron entries.",
        ),
        (
            "System Information",
            "See update counts, uptime, storage, memory, and OS details in one readable summary.",
        ),
        (
            "View Logs",
            "Browse the detailed verbose log captured during update runs.",
        ),
        (
            "Help",
            "Review command-line options, update modes, and log file locations.",
        ),
        ("Exit", "Leave the tool without making changes."),
    ];

    loop {
        match run_menu(
            "Main Menu",
            "Beginner-friendly system maintenance for Ubuntu and Debian systems.",
            &menu_items,
        ) {
            Some(0) => run_updates_menu(),
            Some(1) => manage_schedule(),
            Some(2) => {
                show_information();
                wait_for_enter("Press Enter to return to the main menu...");
            }
            Some(3) => view_verbose_logs(),
            Some(4) => {
                show_help();
                wait_for_enter("Press Enter to return to the main menu...");
            }
            Some(5) | None => {
                tell_user("Exiting. No changes made.");
                log_message("Program exited by user");
                return;
            }
            _ => {}
        }
    }
}

fn run_updates_menu() {
    let menu_items = [
        (
            "Force Update",
            "Runs a full upgrade, cleans packages, and schedules an automatic reboot in 5 minutes.",
        ),
        (
            "All Update",
            "Runs a full upgrade without rebooting. Best default option for most systems.",
        ),
        (
            "Critical Update",
            "Installs security-focused upgrades with no automatic reboot.",
        ),
        ("Back", "Return to the main menu."),
    ];

    loop {
        let state = AppState { dry_run: false };

        match run_menu(
            "Run Updates",
            "Pick the maintenance mode that matches your comfort level and reboot needs.",
            &menu_items,
        ) {
            Some(0) => force_update(&state),
            Some(1) => all_update(&state),
            Some(2) => critical_update(&state),
            Some(3) | None => return,
            _ => {}
        }
    }
}

fn show_help() {
    show_banner(
        "Help",
        "Interactive mode is for guided use; flags are available for repeatable command-line runs.",
    );
    section_heading("Usage");
    println!("  sudo ubuntu-maintenance");
    println!("  sudo ubuntu-maintenance [options]");
    println!();
    section_heading("Options");
    println!("  -f, --force       Full update with an automatic reboot");
    println!("  -a, --all         Full update without rebooting");
    println!("  -c, --critical    Security-focused update only");
    println!("  -i, --info        Show current system information");
    println!("  -d, --dry-run     Preview commands without making changes");
    println!("  -h, --help        Show command help");
    println!();
    section_heading("Recommended flow");
    println!("  1. Start with `sudo ubuntu-maintenance --dry-run -a`");
    println!("  2. Review the plan and available updates");
    println!("  3. Run `sudo ubuntu-maintenance -a` when ready");
    println!();
    section_heading("Log files");
    println!("  Summary log : {}", get_log_path().yellow());
    println!(
        "  Verbose log : {}",
        logger::get_verbose_log_path().yellow()
    );
}

fn show_information() {
    let snapshot = get_system_info_snapshot();

    show_banner(
        "System Information",
        "This summary is cached briefly so repeat visits are fast.",
    );
    section_heading("Identity");
    println!("  Hostname     {}", snapshot.hostname);
    println!("  Operating OS {}", snapshot.os);
    println!("  Kernel       {}", snapshot.kernel);
    println!();
    section_heading("Health Summary");
    println!(
        "  Updates      {} available, {} security-related",
        snapshot.updates_count.to_string().green().bold(),
        snapshot.security_count.to_string().yellow().bold()
    );
    println!("  Disk         {}", snapshot.disk);
    println!("  Memory       {}", snapshot.memory);
    println!();
    section_heading("Activity");
    println!("  Uptime       {}", snapshot.uptime);
    println!("  Last reboot  {}", snapshot.last_reboot);
    println!("  Refreshed    {}", snapshot.fetched_at.dimmed());
    println!();
    log_message("System information displayed");
}

fn safe_reboot(delay_minutes: i32) {
    tell_user("");
    println!("\n{}", "==== SYSTEM REBOOT SCHEDULED ====".blue().bold());

    println!("\nCurrent system time:");
    custom_date_formatted();

    println!("\nSystem will reboot in {} minutes", delay_minutes);

    // Schedule the reboot silently
    let reboot_command = format!(
        "sudo shutdown -r +{} \"System update complete. Rebooting in {} minutes.\"",
        delay_minutes, delay_minutes
    );

    let result = Command::new("sh")
        .arg("-c")
        .arg(&reboot_command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Give shutdown command time to send broadcast message
    thread::sleep(Duration::from_millis(500));

    if result.is_ok() && result.unwrap().success() {
        log_message("System reboot scheduled");

        // Clear screen to hide broadcast messages
        clear_screen();

        // Redisplay clean interface
        println!("\n{}", "==== SYSTEM REBOOT SCHEDULED ====".blue().bold());
        println!("\nCurrent system time:");
        custom_date_formatted();
        println!("\nSystem will reboot in {} minutes", delay_minutes);

        // Show countdown with cancel option
        let seconds = (delay_minutes * 60) as u32;
        let completed =
            countdown_with_cancel(seconds, "System will reboot when timer reaches 0:00");

        if !completed {
            log_message("Reboot cancelled by user");

            // Clear screen again to hide shutdown cancel broadcast
            thread::sleep(Duration::from_millis(500));
            clear_screen();

            println!();
            success_message("✓ Reboot successfully cancelled");
            println!("\nThe system will NOT reboot automatically.");
            println!("You can manually reboot later using: sudo reboot");
            tell_user("");
            tell_user("Press Enter to return to menu...");
            let _ = get_input();
        }
    } else {
        error_message("ERROR: Failed to schedule reboot. Please reboot manually.");
        log_message("ERROR: Failed to schedule reboot");
        tell_user("");
        tell_user("Press Enter to return to menu...");
        let _ = get_input();
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
        tell_user("Press Enter to return to menu...");
        let _ = get_input();
        return;
    }

    clear_screen();
    println!("\n{}\n", "=== FORCE UPDATE (WITH REBOOT) ===".blue().bold());

    tell_system_with_verbose("sudo apt update", "Updating package lists").ok();
    invalidate_system_info_cache();

    println!();
    tell_system_with_verbose(
        "sudo apt full-upgrade -y",
        "Upgrading packages (this may take several minutes)",
    )
    .ok();

    println!();
    tell_system_with_verbose("sudo apt autoremove -y", "Removing obsolete packages").ok();

    println!();
    tell_system_with_verbose("sudo apt autoclean", "Cleaning package cache").ok();

    println!();
    success_message("✓ All updates completed successfully!");

    tell_user("");
    tell_user("Update completed at:");
    custom_date_formatted();

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
        tell_user("  2. sudo apt full-upgrade -y");
        tell_user("  3. sudo apt autoremove -y");
        tell_user("  4. sudo apt autoclean");
        tell_user("");
        tell_user("Press Enter to return to menu...");
        let _ = get_input();
        return;
    }

    clear_screen();
    println!("\n{}\n", "=== ALL UPDATE (NO REBOOT) ===".blue().bold());

    tell_system_with_verbose("sudo apt update", "Updating package lists").ok();
    invalidate_system_info_cache();

    println!();
    tell_system_with_verbose(
        "sudo apt full-upgrade -y",
        "Upgrading packages (this may take several minutes)",
    )
    .ok();

    println!();
    tell_system_with_verbose("sudo apt autoremove -y", "Removing obsolete packages").ok();

    println!();
    tell_system_with_verbose("sudo apt autoclean", "Cleaning package cache").ok();

    println!();
    success_message("✓ All updates completed successfully!");

    tell_user("");
    tell_user("Update completed at:");
    custom_date_formatted();

    tell_user("");
    tell_user("A system reboot is recommended to ensure all updates take effect.");
    tell_user("You can reboot manually using: sudo reboot");

    tell_user("");
    tell_user("Press Enter to return to menu...");
    let _ = get_input();

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
        tell_user("Press Enter to return to menu...");
        let _ = get_input();
        return;
    }

    clear_screen();
    println!(
        "\n{}\n",
        "=== CRITICAL UPDATE (SECURITY ONLY) ===".blue().bold()
    );

    tell_system_with_verbose("sudo apt update", "Updating package lists").ok();
    invalidate_system_info_cache();

    println!();
    tell_system_with_verbose("sudo apt upgrade -y", "Installing security updates").ok();

    println!();
    tell_system_with_verbose("sudo apt autoclean", "Cleaning package cache").ok();

    println!();
    success_message("✓ Security updates completed successfully!");

    tell_user("");
    tell_user("Update completed at:");
    custom_date_formatted();

    tell_user("");
    tell_user("A system reboot is recommended to ensure all updates take effect.");
    tell_user("You can reboot manually using: sudo reboot");

    tell_user("");
    tell_user("Press Enter to return to menu...");
    let _ = get_input();

    log_message("=== CRITICAL UPDATE completed ===");
}

fn manage_schedule() {
    let menu_items = [
        (
            "View Current Schedule",
            "See each scheduled job with a readable explanation of when it runs and what it does.",
        ),
        (
            "Add a New Schedule",
            "Create a daily, weekly, or weekday automation rule.",
        ),
        (
            "Remove All Schedules",
            "Delete every `ubuntu-maintenance` cron entry from the current user's crontab.",
        ),
        ("Back", "Return to the main menu."),
    ];

    loop {
        match run_menu(
            "Schedule Management",
            "Automate maintenance without editing crontab by hand.",
            &menu_items,
        ) {
            Some(0) => {
                show_banner(
                    "Schedule Management",
                    "Automate maintenance without editing crontab by hand.",
                );
                let _ = show_current_schedule();
                wait_for_enter("Press Enter to continue...");
            }
            Some(1) => add_schedule_menu(),
            Some(2) => remove_schedule_menu(),
            Some(3) | None => return,
            _ => {}
        }
    }
}

fn add_schedule_menu() {
    let frequency = match run_menu(
        "Add Schedule",
        "Step 1 of 2: choose when automated maintenance should run.",
        &[
            ("Daily", "Every day at 2:00 AM."),
            ("Weekly", "Every Sunday at 3:00 AM."),
            ("Weekdays", "Monday through Friday at 2:00 AM."),
            ("Cancel", "Return without creating a schedule."),
        ],
    ) {
        Some(0) => "daily",
        Some(1) => "weekly",
        Some(2) => "weekdays",
        _ => return,
    };

    let mode = match run_menu(
        "Add Schedule",
        "Step 2 of 2: choose what kind of maintenance this job should perform.",
        &[
            (
                "All Updates",
                "Recommended. Installs all package updates and does not reboot automatically.",
            ),
            ("Critical Updates", "Security-focused upgrades only."),
            (
                "Force Update",
                "Full update followed by an automatic reboot. Use only when planned.",
            ),
            ("Cancel", "Return without creating a schedule."),
        ],
    ) {
        Some(0) => "all",
        Some(1) => "critical",
        Some(2) => "force",
        _ => return,
    };

    // Confirm
    show_banner(
        "Confirm Schedule",
        "Review the automation rule before it is written to cron.",
    );
    println!("You are about to schedule:");
    println!("  Frequency: {}", frequency);
    println!("  Mode: {} updates", mode);

    if mode == "force" {
        println!();
        warning_message("WARNING: Force mode will automatically reboot your server!");
        println!(
            "This should only be used if you have redundancy or scheduled maintenance windows."
        );
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
                wait_for_enter("Press Enter to continue...");
            }
            Err(_) => {
                tell_user("");
                error_message("Failed to add schedule. Check logs for details.");
                wait_for_enter("Press Enter to continue...");
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
                        wait_for_enter("Press Enter to continue...");
                    }
                    Err(_) => {
                        error_message("Failed to remove schedules. Check logs for details.");
                        wait_for_enter("Press Enter to continue...");
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
}

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
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use utils::*;

const VERSION: &str = "3.1.10";
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
        "Run Updates...",
        "Manage Update Schedule...",
        "System Information",
        "View Logs",
        "Help",
        "", // Blank line
        "Exit",
    ];

    let mut selected = 0;

    loop {
        clear_screen();

        // Header
        println!("\n{}\n", "=== UBUNTU MAINTENANCE ===".blue().bold());

        // Menu items
        for (i, item) in menu_items.iter().enumerate() {
            if item.is_empty() {
                println!(); // Blank line
            } else if i == selected {
                println!("{} {}", "*".green().bold(), item.green());
            } else {
                println!("  {}", item);
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
                        } else {
                            // Wrap to bottom
                            selected = menu_items.len() - 1;
                        }
                        // Skip blank lines
                        while menu_items[selected].is_empty() {
                            if selected > 0 {
                                selected -= 1;
                            } else {
                                selected = menu_items.len() - 1;
                            }
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        } else {
                            // Wrap to top
                            selected = 0;
                        }
                        // Skip blank lines
                        while menu_items[selected].is_empty() {
                            if selected < menu_items.len() - 1 {
                                selected += 1;
                            } else {
                                selected = 0;
                            }
                        }
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(selected);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Some(6); // Exit option (last item)
                    }
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        // Process selection
        if let Some(idx) = choice {
            match idx {
                0 => {
                    run_updates_menu();
                }
                1 => {
                    manage_schedule();
                }
                2 => {
                    show_information();
                    tell_user("");
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
                }
                3 => {
                    view_verbose_logs();
                }
                4 => {
                    show_help();
                    tell_user("Press Enter to return to menu...");
                    let _ = get_input();
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

fn run_updates_menu() {
    let menu_items = vec![
        "Force Update (with reboot)",
        "All Update (no reboot)",
        "Critical Update (security only)",
        "",
        "Return to main menu",
    ];

    let mut selected = 0;

    loop {
        clear_screen();
        println!("\n{}\n", "=== RUN UPDATES ===".blue().bold());

        // Menu items
        for (i, item) in menu_items.iter().enumerate() {
            if item.is_empty() {
                println!();
            } else if i == selected {
                println!("{} {}", "*".green().bold(), item.green());
            } else {
                println!("  {}", item);
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
                        } else {
                            selected = menu_items.len() - 1;
                        }
                        while menu_items[selected].is_empty() {
                            if selected > 0 {
                                selected -= 1;
                            } else {
                                selected = menu_items.len() - 1;
                            }
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
                        }
                        while menu_items[selected].is_empty() {
                            if selected < menu_items.len() - 1 {
                                selected += 1;
                            } else {
                                selected = 0;
                            }
                        }
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(selected);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Some(4); // Return option
                    }
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

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
                4 => return,
                _ => {}
            }
        }
    }
}

fn show_help() {
    clear_screen();
    println!("\n{}\n", "=== HELP ===".blue().bold());
    println!("Usage: sudo ubuntu-maintenance [options]\n");
    println!("Options:");
    println!("  -f, --force       Force update with reboot");
    println!("  -a, --all         All updates without reboot");
    println!("  -c, --critical    Critical security updates only");
    println!("  -i, --info        Display system information");
    println!("  -d, --dry-run     Preview updates without applying");
    println!("  -h, --help        Display this help message\n");
    println!("Interactive Mode: Run without arguments for interactive menu\n");
    println!("Examples:");
    println!("  sudo ubuntu-maintenance              # Interactive mode");
    println!("  sudo ubuntu-maintenance -a           # Run all updates");
    println!("  sudo ubuntu-maintenance --dry-run -a # Preview updates\n");
    println!("Logs: {}\n", get_log_path());
}

fn show_information() {
    clear_screen();
    println!("\n{}\n", "=== SYSTEM INFORMATION ===".blue().bold());

    // Hostname and kernel
    if let Ok(output) = Command::new("uname").arg("-n").output() {
        print!("Hostname: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        println!(" | Kernel: {}", String::from_utf8_lossy(&output.stdout).trim());
    } else {
        println!();
    }

    // OS version
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("lsb_release -d 2>/dev/null | cut -f2")
        .output()
    {
        println!("OS: {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    // Uptime and last reboot
    if let Ok(output) = Command::new("uptime").arg("-p").output() {
        print!("Uptime: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("who -b | awk '{print $3, $4}'")
        .output()
    {
        println!(" | Last reboot: {}", String::from_utf8_lossy(&output.stdout).trim());
    } else {
        println!();
    }

    println!();

    // Package updates
    let updates_count = Command::new("sh")
        .arg("-c")
        .arg("apt list --upgradable 2>/dev/null | grep -v 'Listing...' | wc -l")
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<i32>().ok())
        .unwrap_or(0);

    let security_count = Command::new("sh")
        .arg("-c")
        .arg("apt list --upgradable 2>/dev/null | grep -i security | wc -l")
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<i32>().ok())
        .unwrap_or(0);

    println!("Updates: {} available ({} security)", updates_count, security_count);

    // Disk usage (using -H for SI units: GB instead of GiB)
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("df -H / | tail -1 | awk '{print $5 \" used | \" $4 \" free | \" $2 \" total\"}'")
        .output()
    {
        println!("Disk (root): {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    // Memory usage (using --si for SI units: GB instead of GiB)
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("free -h --si | awk 'NR==2 {print $3 \" used | \" $7 \" available | \" $2 \" total\"}'")
        .output()
    {
        println!("Memory: {}", String::from_utf8_lossy(&output.stdout).trim());
    }

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
        let completed = countdown_with_cancel(seconds, "System will reboot when timer reaches 0:00");

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

    println!();
    tell_system_with_verbose("sudo apt full-upgrade -y", "Upgrading packages (this may take several minutes)").ok();

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

    println!();
    tell_system_with_verbose("sudo apt full-upgrade -y", "Upgrading packages (this may take several minutes)").ok();

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
    println!("\n{}\n", "=== CRITICAL UPDATE (SECURITY ONLY) ===".blue().bold());

    tell_system_with_verbose("sudo apt update", "Updating package lists").ok();

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
    let menu_items = vec![
        "View current schedule",
        "Add a new schedule",
        "Remove all schedules",
        "",
        "Return to main menu",
    ];

    let mut selected = 0;

    loop {
        clear_screen();
        println!("\n{}\n", "=== MANAGE UPDATE SCHEDULE ===".blue().bold());

        // Menu items
        for (i, item) in menu_items.iter().enumerate() {
            if item.is_empty() {
                println!();
            } else if i == selected {
                println!("{} {}", "*".green().bold(), item.green());
            } else {
                println!("  {}", item);
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
                        } else {
                            selected = menu_items.len() - 1;
                        }
                        while menu_items[selected].is_empty() {
                            if selected > 0 {
                                selected -= 1;
                            } else {
                                selected = menu_items.len() - 1;
                            }
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
                        }
                        while menu_items[selected].is_empty() {
                            if selected < menu_items.len() - 1 {
                                selected += 1;
                            } else {
                                selected = 0;
                            }
                        }
                        break None;
                    }
                    KeyCode::Enter => {
                        break Some(selected);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Some(4);
                    }
                    _ => {}
                }
            }
        };

        let _ = disable_raw_mode();

        if let Some(idx) = choice {
            match idx {
                0 => {
                    clear_screen();
                    let _ = show_current_schedule();
                    tell_user("Press Enter to continue...");
                    let _ = get_input();
                }
                1 => add_schedule_menu(),
                2 => remove_schedule_menu(),
                4 => return,
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
                println!("{} {}", "*".green().bold(), item.green());
            } else {
                println!("  {}", item);
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
                        } else {
                            selected = freq_items.len() - 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < freq_items.len() - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
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
                println!("{} {}", "*".green().bold(), item.green());
            } else {
                println!("  {}", item);
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
                        } else {
                            selected = mode_items.len() - 1;
                        }
                        break None;
                    }
                    KeyCode::Down => {
                        if selected < mode_items.len() - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
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

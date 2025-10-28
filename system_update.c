/* Production-ready Ubuntu Server Maintenance Tool
   Author: Vincent T. Mossman
   Updated for production use with security hardening

   Compile by:  gcc -o system_update system_update.c utility_functions.c
   Run by:      sudo ./system_update [options]

   Options:
     -f, --force       Force update with reboot
     -a, --all         All updates without reboot
     -c, --critical    Critical security updates only
     -i, --info        Display system information
     -d, --dry-run     Preview updates without applying
     -h, --help        Display this help message
*/

#include "utility_functions.h"
#include <unistd.h>
#include <sys/utsname.h>

// Global configuration
static int dry_run_mode = 0;
static int reboot_delay_minutes = 5;

// Function declarations
void main_menu(void);
void execute(const char* option);
void complete_details(void);
void force_update(void);
void all_update(void);
void critical_update(void);
void show_information(void);
void show_help(void);
int check_sudo(void);
void safe_reboot(int delay_minutes);

// Helper function to check if running with sudo
int check_sudo(void) {
    if (geteuid() != 0) {
        tell_user("ERROR: This program must be run with sudo privileges.");
        tell_user("Usage: sudo ./system_update [options]");
        log_message("ERROR: Attempted to run without sudo");
        return 0;
    }
    return 1;
}

void show_help(void) {
    clear_screen();
    tell_user("=== UBUNTU SERVER MAINTENANCE TOOL ===");
    tell_user("");
    tell_user("Usage: sudo ./system_update [options]");
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
    tell_user("  sudo ./system_update            # Interactive mode");
    tell_user("  sudo ./system_update -a         # Run all updates");
    tell_user("  sudo ./system_update --dry-run -a  # Preview all updates");
    tell_user("");
    tell_user("Logs are written to: /var/log/system_update.log");
    tell_user("");
}

void show_information(void) {
    clear_screen();
    tell_user("=== SYSTEM INFORMATION ===");
    tell_user("");

    // System details
    struct utsname sys_info;
    if (uname(&sys_info) == 0) {
        printf("System:       %s\n", sys_info.sysname);
        printf("Hostname:     %s\n", sys_info.nodename);
        printf("Release:      %s\n", sys_info.release);
        printf("Version:      %s\n", sys_info.version);
        printf("Architecture: %s\n", sys_info.machine);
    }

    tell_user("");
    tell_user("Current Time:");
    custom_date_formatted();

    tell_user("");
    tell_user("=== PACKAGE INFORMATION ===");
    tell_system("apt --version");

    tell_user("");
    tell_user("=== AVAILABLE UPDATES ===");
    tell_system("apt list --upgradable 2>/dev/null | grep -v 'Listing...' | wc -l | xargs echo 'Packages with updates available:'");

    tell_user("");
    tell_user("=== SECURITY UPDATES ===");
    tell_system("apt list --upgradable 2>/dev/null | grep -i security | wc -l | xargs echo 'Security updates available:'");

    tell_user("");
    tell_user("=== DISK USAGE ===");
    tell_system("df -h / | tail -1 | awk '{print \"Root partition: \" $5 \" used of \" $2}'");

    tell_user("");
    tell_user("=== SYSTEM UPTIME ===");
    tell_system("uptime -p");

    tell_user("");
    tell_user("=== LAST REBOOT ===");
    tell_system("who -b");

    tell_user("");
    log_message("System information displayed");
}

void complete_details(void) {
    tell_user("");
    tell_user("==== UPDATE COMPLETE ====");

    // Show APT version
    tell_system("apt --version");

    tell_user("");
    tell_user("Update completed at:");
    custom_date_formatted();

    tell_user("");
    tell_user("A system reboot is recommended to ensure all updates take effect.");
    tell_user("You can reboot now using: sudo shutdown -r +5");

    log_message("Update completed successfully");
}

void safe_reboot(int delay_minutes) {
    char reboot_command[256];

    tell_user("");
    tell_user("==== SCHEDULING SYSTEM REBOOT ====");

    snprintf(reboot_command, sizeof(reboot_command),
             "sudo shutdown -r +%d \"System update complete. Rebooting in %d minutes. Use 'shutdown -c' to cancel.\"",
             delay_minutes, delay_minutes);

    tell_user_custom_formatting("Current system time:", 1, 0);
    custom_date_formatted();

    printf("\nReboot scheduled for %d minutes from now.\n", delay_minutes);
    tell_user("To cancel: sudo shutdown -c");
    tell_user("");

    if (tell_system(reboot_command) == 0) {
        log_message("System reboot scheduled");
        tell_user("Please save your work and logout, or cancel the reboot if needed.");
    } else {
        tell_user("ERROR: Failed to schedule reboot. Please reboot manually.");
        log_message("ERROR: Failed to schedule reboot");
    }
}

void force_update(void) {
    log_message("=== Starting FORCE UPDATE ===");

    if (dry_run_mode) {
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

    /* Full system update with automatic reboot
       1. Update package information
       2. Install all available upgrades (full-upgrade)
       3. Remove obsolete packages
       4. Clean up package cache
       5. Schedule system reboot
    */

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if (tell_system("sudo apt update") != 0) {
        tell_user("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING ALL AVAILABLE PACKAGE UPGRADES ====");
    tell_user("This may take several minutes depending on available updates...");
    if (tell_system("sudo apt full-upgrade -y") != 0) {
        tell_user("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: full-upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== REMOVING OBSOLETE PACKAGES ====");
    tell_system("sudo apt autoremove -y");

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    tell_system("sudo apt autoclean");

    complete_details();

    tell_user("");
    safe_reboot(reboot_delay_minutes);

    log_message("=== FORCE UPDATE completed ===");
}

void all_update(void) {
    log_message("=== Starting ALL UPDATE ===");

    if (dry_run_mode) {
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

    /* Full system update without automatic reboot
       1. Update package information
       2. Install all available upgrades (full-upgrade)
       3. Remove obsolete packages
       4. Clean up package cache
    */

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if (tell_system("sudo apt update") != 0) {
        tell_user("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING ALL AVAILABLE PACKAGE UPGRADES ====");
    tell_user("This may take several minutes depending on available updates...");
    if (tell_system("sudo apt full-upgrade") != 0) {
        tell_user("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: full-upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== REMOVING OBSOLETE PACKAGES ====");
    tell_system("sudo apt autoremove");

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    tell_system("sudo apt autoclean");

    complete_details();

    log_message("=== ALL UPDATE completed ===");
}

void critical_update(void) {
    log_message("=== Starting CRITICAL UPDATE ===");

    if (dry_run_mode) {
        tell_user("=== DRY RUN MODE: Preview of Critical Update ===");
        tell_user("");
        tell_user("The following commands would be executed:");
        tell_user("  1. sudo apt update");
        tell_user("  2. sudo apt upgrade (security updates)");
        tell_user("  3. sudo apt autoclean");
        tell_user("");
        return;
    }

    /* Security and critical updates only
       1. Update package information
       2. Install available upgrades (upgrade, not full-upgrade)
       3. Clean up package cache
    */

    tell_user("==== UPDATING PACKAGE INFORMATION ====");
    if (tell_system("sudo apt update") != 0) {
        tell_user("WARNING: Package update had issues. Check logs.");
    }

    tell_user("");
    tell_user("==== INSTALLING CRITICAL PACKAGE UPGRADES ====");
    tell_user("Installing security and critical updates only...");
    if (tell_system("sudo apt upgrade -y") != 0) {
        tell_user("WARNING: Package upgrade had issues. Check logs.");
        log_message("WARNING: upgrade returned non-zero exit code");
    }

    tell_user("");
    tell_user("==== CLEANING PACKAGE CACHE ====");
    tell_system("sudo apt autoclean");

    complete_details();

    log_message("=== CRITICAL UPDATE completed ===");
}

void main_menu(void) {
    char input[100];
    int choice;

    clear_screen();
    tell_user_custom_formatting("=== UBUNTU SERVER MAINTENANCE TOOL ===", 1, 1);
    tell_user_custom_formatting("1) Force Update (with reboot)", 0, 0);
    tell_user_custom_formatting("2) All Update (no reboot)", 0, 0);
    tell_user_custom_formatting("3) Critical Update (security only)", 0, 0);
    tell_user_custom_formatting("4) System Information", 0, 0);
    tell_user_custom_formatting("5) Help", 0, 0);
    tell_user_custom_formatting("0) Exit", 0, 1);
    tell_user_no_formatting("Enter choice (0-5): ");

    if (fgets(input, sizeof(input), stdin) == NULL) {
        tell_user("ERROR: Failed to read input");
        log_message("ERROR: Failed to read menu input");
        return;
    }

    // Remove newline
    input[strcspn(input, "\n")] = '\0';

    choice = validate_menu_input(input, 0, 5);

    if (choice == -1) {
        tell_user("");
        tell_user("Invalid choice. Please enter a number between 0 and 5.");
        tell_user("");
        sleep(2);
        main_menu();  // Show menu again
        return;
    }

    switch (choice) {
        case 1:
            force_update();
            break;
        case 2:
            all_update();
            break;
        case 3:
            critical_update();
            break;
        case 4:
            show_information();
            tell_user("");
            tell_user("Press Enter to return to menu...");
            getchar();
            main_menu();
            break;
        case 5:
            show_help();
            tell_user("Press Enter to return to menu...");
            getchar();
            main_menu();
            break;
        case 0:
            tell_user("Exiting. No changes made.");
            log_message("Program exited by user");
            break;
        default:
            tell_user("Invalid choice.");
            sleep(1);
            main_menu();
    }
}

void execute(const char* option) {
    if (option == NULL) {
        tell_user("ERROR: No option provided");
        return;
    }

    // Check for long options
    if (strcmp(option, "--force") == 0 || strcmp(option, "-f") == 0) {
        force_update();
    } else if (strcmp(option, "--all") == 0 || strcmp(option, "-a") == 0) {
        all_update();
    } else if (strcmp(option, "--critical") == 0 || strcmp(option, "-c") == 0) {
        critical_update();
    } else if (strcmp(option, "--info") == 0 || strcmp(option, "-i") == 0) {
        show_information();
    } else if (strcmp(option, "--help") == 0 || strcmp(option, "-h") == 0) {
        show_help();
    } else if (strcmp(option, "--dry-run") == 0 || strcmp(option, "-d") == 0) {
        dry_run_mode = 1;
        tell_user("DRY RUN MODE ENABLED - No changes will be made");
        log_message("Dry run mode enabled");
    } else {
        tell_user("ERROR: Invalid option");
        tell_user("Use -h or --help for usage information");
    }
}

int main(int argc, char* argv[]) {
    // Initialize logging
    log_message("=== System Update Tool Started ===");

    // Parse command line arguments
    if (argc == 1) {
        // No arguments - show interactive menu
        if (!check_sudo()) {
            return EXIT_FAILURE;
        }
        main_menu();
    } else {
        // Process command line arguments
        int dry_run_found = 0;
        int command_found = 0;

        // First pass: check for dry-run flag
        for (int i = 1; i < argc; i++) {
            if (strcmp(argv[i], "--dry-run") == 0 || strcmp(argv[i], "-d") == 0) {
                dry_run_mode = 1;
                dry_run_found = 1;
                tell_user("DRY RUN MODE ENABLED - No changes will be made");
                log_message("Dry run mode enabled");
            }
        }

        // Second pass: execute commands
        for (int i = 1; i < argc; i++) {
            if (strcmp(argv[i], "--dry-run") == 0 || strcmp(argv[i], "-d") == 0) {
                continue;  // Already processed
            }

            // Help doesn't require sudo
            if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
                show_help();
                command_found = 1;
                continue;
            }

            // Info doesn't require sudo for most info
            if (strcmp(argv[i], "--info") == 0 || strcmp(argv[i], "-i") == 0) {
                show_information();
                command_found = 1;
                continue;
            }

            // Other commands require sudo
            if (!check_sudo()) {
                return EXIT_FAILURE;
            }

            execute(argv[i]);
            command_found = 1;
        }

        if (!command_found && dry_run_found) {
            tell_user("ERROR: --dry-run must be used with another option");
            tell_user("Example: sudo ./system_update --dry-run -a");
            show_help();
            return EXIT_FAILURE;
        }

        if (!command_found) {
            tell_user("ERROR: Invalid arguments");
            show_help();
            return EXIT_FAILURE;
        }
    }

    log_message("=== System Update Tool Finished ===");
    return EXIT_SUCCESS;
}

/* A collection of useful functions for system maintenance.
   Author: Vincent T. Mossman
   Updated for production use with security hardening
*/

#include "utility_functions.h"

#define LOG_FILE "/var/log/system_update.log"
#define MAX_COMMAND_LENGTH 1024

// Helper function to safely allocate memory with error checking
static void* safe_malloc(size_t size) {
    void* ptr = malloc(size);
    if (ptr == NULL) {
        fprintf(stderr, "ERROR: Memory allocation failed for %zu bytes\n", size);
        log_message("CRITICAL: Memory allocation failure");
    }
    return ptr;
}

// Console output functions
void tell_user(const char* message) {
    if (message == NULL) return;

    printf("\n\n%s\n", message);
    fflush(stdout);
}

void tell_user_custom_formatting(const char* message, int leading_nl, int trailing_nl) {
    if (message == NULL) return;

    for (int i = 0; i < leading_nl; i++) {
        printf("\n");
    }
    printf("%s", message);
    for (int i = 0; i < trailing_nl; i++) {
        printf("\n");
    }
    fflush(stdout);
}

void tell_user_no_formatting(const char* message) {
    if (message == NULL) return;

    printf("%s", message);
    fflush(stdout);
}

void clear_screen(void) {
    system("clear");
}

void cls(void) {
    clear_screen();
}

int tell_system(const char* command) {
    if (command == NULL) {
        fprintf(stderr, "ERROR: NULL command provided to tell_system\n");
        return EXIT_FAILURE;
    }

    log_command(command, 0);
    int exit_code = system(command);

    if (exit_code != 0) {
        fprintf(stderr, "WARNING: Command returned exit code %d: %s\n",
                exit_code, command);
        log_command(command, exit_code);
    }

    return exit_code;
}

// String manipulation functions
void prepend(char* s, const char* t, size_t max_size) {
    if (s == NULL || t == NULL || max_size == 0) return;

    size_t len_t = strlen(t);
    size_t len_s = strlen(s);

    // Check if there's enough space
    if (len_t + len_s + 1 > max_size) {
        fprintf(stderr, "ERROR: Buffer overflow prevented in prepend\n");
        return;
    }

    memmove(s + len_t, s, len_s + 1);
    memcpy(s, t, len_t);
}

void append(char* s, const char* t, size_t max_size) {
    if (s == NULL || t == NULL || max_size == 0) return;

    size_t len_t = strlen(t);
    size_t len_s = strlen(s);

    // Check if there's enough space
    if (len_s + len_t + 1 > max_size) {
        fprintf(stderr, "ERROR: Buffer overflow prevented in append\n");
        return;
    }

    memcpy(s + len_s, t, len_t);
    s[len_s + len_t] = '\0';
}

// Date/Time functions
void custom_date_formatted(void) {
    time_t t = time(NULL);
    struct tm timeinfo = *localtime(&t);

    int int_day = timeinfo.tm_mday;
    char* suffix = number_suffix(int_day);

    if (suffix == NULL) {
        fprintf(stderr, "ERROR: Failed to get number suffix\n");
        return;
    }

    // Use strftime for safer date formatting
    char date_buffer[256];
    char time_buffer[256];

    strftime(time_buffer, sizeof(time_buffer), "%I:%M%p", &timeinfo);
    strftime(date_buffer, sizeof(date_buffer), "%A, %B", &timeinfo);

    printf("%s on %s, the %d%s, %d\n",
           time_buffer, date_buffer, int_day, suffix, timeinfo.tm_year + 1900);

    // Get more precise time
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) == 0) {
        printf("At precisely %ld seconds and %ld nanoseconds\n",
               ts.tv_sec % 60, ts.tv_nsec);
    }

    free(suffix);
}

char* number_suffix(int x) {
    char* suffix = safe_malloc(3);
    if (suffix == NULL) return NULL;

    if (x % 10 == 1 && x % 100 != 11) {
        strcpy(suffix, "st");
    } else if (x % 10 == 2 && x % 100 != 12) {
        strcpy(suffix, "nd");
    } else if (x % 10 == 3 && x % 100 != 13) {
        strcpy(suffix, "rd");
    } else {
        strcpy(suffix, "th");
    }

    return suffix;
}

// Conversion functions - FIXED: Now uses sprintf instead of massive switch
int char_to_int(const char* s) {
    if (s == NULL) return 0;

    int x = 0;
    if (sscanf(s, "%d", &x) != 1) {
        fprintf(stderr, "ERROR: Failed to convert '%s' to integer\n", s);
        return 0;
    }
    return x;
}

char* int_to_char(int x) {
    // Allocate enough space for any 32-bit integer plus null terminator
    char* convert = safe_malloc(12);
    if (convert == NULL) return NULL;

    snprintf(convert, 12, "%d", x);
    return convert;
}

// Input validation
int validate_menu_input(const char* input, int min, int max) {
    if (input == NULL) return -1;

    char* endptr;
    errno = 0;
    long val = strtol(input, &endptr, 10);

    // Check for conversion errors
    if (errno != 0 || endptr == input || *endptr != '\0') {
        return -1;
    }

    // Check range
    if (val < min || val > max) {
        return -1;
    }

    return (int)val;
}

// Logging functions
void log_message(const char* message) {
    if (message == NULL) return;

    FILE* log_file = fopen(LOG_FILE, "a");
    if (log_file == NULL) {
        // If we can't open the log file, try a fallback location
        log_file = fopen("/tmp/system_update.log", "a");
        if (log_file == NULL) {
            fprintf(stderr, "WARNING: Cannot open log file\n");
            return;
        }
    }

    time_t now = time(NULL);
    char* time_str = ctime(&now);
    if (time_str != NULL) {
        // Remove newline from ctime
        time_str[strcspn(time_str, "\n")] = '\0';
        fprintf(log_file, "[%s] %s\n", time_str, message);
    } else {
        fprintf(log_file, "[UNKNOWN TIME] %s\n", message);
    }

    fclose(log_file);
}

void log_command(const char* command, int exit_code) {
    if (command == NULL) return;

    char log_buffer[MAX_COMMAND_LENGTH + 100];

    if (exit_code == 0) {
        snprintf(log_buffer, sizeof(log_buffer),
                "CMD: %s", command);
    } else {
        snprintf(log_buffer, sizeof(log_buffer),
                "CMD FAILED (exit %d): %s", exit_code, command);
    }

    log_message(log_buffer);
}

// Schedule management functions

int has_existing_schedule(void) {
    FILE *fp;
    char line[1024];
    int found = 0;

    // Use crontab -l to list current user's cron jobs
    fp = popen("crontab -l 2>/dev/null | grep -c system_update", "r");
    if (fp == NULL) {
        return -1;
    }

    if (fgets(line, sizeof(line), fp) != NULL) {
        int count = atoi(line);
        found = (count > 0) ? 1 : 0;
    }

    pclose(fp);
    return found;
}

void show_current_schedule(void) {
    FILE *fp;
    char line[1024];
    int count = 0;

    tell_user("=== CURRENT UPDATE SCHEDULES ===");
    tell_user("");

    // List all cron jobs containing system_update
    fp = popen("crontab -l 2>/dev/null | grep system_update", "r");
    if (fp == NULL) {
        tell_user("ERROR: Unable to read cron schedule");
        return;
    }

    while (fgets(line, sizeof(line), fp) != NULL) {
        // Remove trailing newline
        line[strcspn(line, "\n")] = '\0';

        // Skip comments
        if (line[0] == '#') {
            continue;
        }

        count++;

        // Parse and display in a user-friendly format
        printf("Schedule %d: %s\n", count, line);

        // Try to interpret the schedule
        if (strstr(line, "0 2 * * *") != NULL) {
            printf("  → Runs daily at 2:00 AM\n");
        } else if (strstr(line, "0 3 * * 0") != NULL) {
            printf("  → Runs weekly on Sunday at 3:00 AM\n");
        } else if (strstr(line, "0 2 * * 1-5") != NULL) {
            printf("  → Runs weekdays at 2:00 AM\n");
        }

        if (strstr(line, "-a") != NULL || strstr(line, "--all") != NULL) {
            printf("  → Mode: All updates (no reboot)\n");
        } else if (strstr(line, "-c") != NULL || strstr(line, "--critical") != NULL) {
            printf("  → Mode: Critical/security updates only\n");
        } else if (strstr(line, "-f") != NULL || strstr(line, "--force") != NULL) {
            printf("  → Mode: Force update with reboot\n");
        }

        printf("\n");
    }

    pclose(fp);

    if (count == 0) {
        tell_user("No scheduled updates found.");
        tell_user("");
        tell_user("You can add a schedule from the main menu option 6.");
    } else {
        printf("Total schedules: %d\n", count);
    }

    tell_user("");
}

int add_schedule(const char* frequency, const char* mode) {
    if (frequency == NULL || mode == NULL) {
        tell_user("ERROR: Invalid parameters for add_schedule");
        return -1;
    }

    // Build the cron schedule
    char cron_schedule[20];
    char cron_command[512];
    char full_cron_line[600];

    // Determine cron time based on frequency
    if (strcmp(frequency, "daily") == 0) {
        strcpy(cron_schedule, "0 2 * * *");  // 2 AM daily
    } else if (strcmp(frequency, "weekly") == 0) {
        strcpy(cron_schedule, "0 3 * * 0");  // 3 AM every Sunday
    } else if (strcmp(frequency, "weekdays") == 0) {
        strcpy(cron_schedule, "0 2 * * 1-5");  // 2 AM Monday-Friday
    } else {
        tell_user("ERROR: Invalid frequency. Use 'daily', 'weekly', or 'weekdays'");
        return -1;
    }

    // Determine the command mode flag
    char mode_flag[20];
    if (strcmp(mode, "all") == 0) {
        strcpy(mode_flag, "-a");
    } else if (strcmp(mode, "critical") == 0) {
        strcpy(mode_flag, "-c");
    } else if (strcmp(mode, "force") == 0) {
        strcpy(mode_flag, "-f");
    } else {
        tell_user("ERROR: Invalid mode. Use 'all', 'critical', or 'force'");
        return -1;
    }

    // Get the full path to system_update
    char binary_path[256];
    FILE *fp = popen("which system_update 2>/dev/null", "r");
    if (fp != NULL && fgets(binary_path, sizeof(binary_path), fp) != NULL) {
        binary_path[strcspn(binary_path, "\n")] = '\0';
        pclose(fp);
    } else {
        if (fp) pclose(fp);
        // Default paths to try
        if (access("/usr/bin/system_update", X_OK) == 0) {
            strcpy(binary_path, "/usr/bin/system_update");
        } else if (access("/usr/local/bin/system_update", X_OK) == 0) {
            strcpy(binary_path, "/usr/local/bin/system_update");
        } else {
            tell_user("ERROR: Cannot find system_update binary");
            return -1;
        }
    }

    // Build the full cron command with output redirection
    snprintf(cron_command, sizeof(cron_command),
             "%s %s >> /var/log/automated_updates.log 2>&1",
             binary_path, mode_flag);

    // Build the complete cron line
    snprintf(full_cron_line, sizeof(full_cron_line),
             "%s %s", cron_schedule, cron_command);

    // Create a temporary file with current crontab + new line
    char temp_file[] = "/tmp/crontab.XXXXXX";
    int fd = mkstemp(temp_file);
    if (fd == -1) {
        tell_user("ERROR: Cannot create temporary file");
        return -1;
    }

    FILE *temp_fp = fdopen(fd, "w");
    if (temp_fp == NULL) {
        close(fd);
        unlink(temp_file);
        tell_user("ERROR: Cannot open temporary file");
        return -1;
    }

    // Write existing crontab if any
    fp = popen("crontab -l 2>/dev/null", "r");
    if (fp != NULL) {
        char line[1024];
        while (fgets(line, sizeof(line), fp) != NULL) {
            fputs(line, temp_fp);
        }
        pclose(fp);
    }

    // Add new cron job
    fprintf(temp_fp, "# system_update - automated %s %s updates\n", frequency, mode);
    fprintf(temp_fp, "%s\n", full_cron_line);
    fclose(temp_fp);

    // Install new crontab
    char install_cmd[512];
    snprintf(install_cmd, sizeof(install_cmd), "crontab %s", temp_file);

    int result = system(install_cmd);
    unlink(temp_file);

    if (result != 0) {
        tell_user("ERROR: Failed to install crontab");
        log_message("ERROR: Failed to add cron schedule");
        return -1;
    }

    tell_user("SUCCESS: Schedule added successfully!");

    char log_msg[256];
    snprintf(log_msg, sizeof(log_msg),
             "Added cron schedule: %s updates %s", mode, frequency);
    log_message(log_msg);

    return 0;
}

int remove_all_schedules(void) {
    FILE *fp;
    char line[1024];
    int removed = 0;

    // Create temporary file
    char temp_file[] = "/tmp/crontab.XXXXXX";
    int fd = mkstemp(temp_file);
    if (fd == -1) {
        tell_user("ERROR: Cannot create temporary file");
        return -1;
    }

    FILE *temp_fp = fdopen(fd, "w");
    if (temp_fp == NULL) {
        close(fd);
        unlink(temp_file);
        tell_user("ERROR: Cannot open temporary file");
        return -1;
    }

    // Read current crontab and filter out system_update entries
    fp = popen("crontab -l 2>/dev/null", "r");
    if (fp != NULL) {
        int skip_next = 0;

        while (fgets(line, sizeof(line), fp) != NULL) {
            // If previous line was a system_update comment, skip this line too
            if (skip_next) {
                skip_next = 0;
                removed++;
                continue;
            }

            // Check if this is a system_update comment
            if (strstr(line, "# system_update") != NULL) {
                skip_next = 1;
                continue;
            }

            // Check if line contains system_update command
            if (strstr(line, "system_update") != NULL) {
                removed++;
                continue;
            }

            // Keep this line
            fputs(line, temp_fp);
        }
        pclose(fp);
    }

    fclose(temp_fp);

    if (removed == 0) {
        unlink(temp_file);
        tell_user("No system_update schedules found to remove.");
        return 0;
    }

    // Install new crontab (without system_update entries)
    char install_cmd[512];
    snprintf(install_cmd, sizeof(install_cmd), "crontab %s", temp_file);

    int result = system(install_cmd);
    unlink(temp_file);

    if (result != 0) {
        tell_user("ERROR: Failed to update crontab");
        log_message("ERROR: Failed to remove cron schedules");
        return -1;
    }

    printf("\nSUCCESS: Removed %d schedule(s)\n\n", removed);

    char log_msg[256];
    snprintf(log_msg, sizeof(log_msg), "Removed %d cron schedule(s)", removed);
    log_message(log_msg);

    return 0;
}

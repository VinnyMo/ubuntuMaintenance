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

/* A collection of useful functions for system maintenance.
   Author: Vincent T. Mossman
   Updated for production use with security hardening
*/

#ifndef UTILITY_FUNCTIONS_H
#define UTILITY_FUNCTIONS_H

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <errno.h>

// Return codes for error handling
#define SUCCESS 0
#define ERROR_MALLOC -1
#define ERROR_COMMAND -2
#define ERROR_INPUT -3

// Console output functions
void tell_user(const char* message);
  /* tell_user sends message to the console for user to read. Default
      formatting - two newlines before message, one newline following.
      */

void tell_user_custom_formatting(const char* message, int leading_nl, int trailing_nl);
  /* tell_user sends message to the console for user to read. Allows
      custom formatting; leading_nl newlines before message,
      trailing_nl newlines following.
      */

void tell_user_no_formatting(const char* message);
  /* tell_user_no_formatting sends message to the console for user to
      read. There is no formatting; no newlines.
      */

void clear_screen(void);
  /* clear_screen clears the console.
     */

void cls(void);
  /* cls is the same as clear_screen.
     */

int tell_system(const char* command);
  /* tell_system sends command to system to be executed.
      Returns: EXIT_SUCCESS on success, EXIT_FAILURE on error
      */

// String manipulation functions
void prepend(char* s, const char* t, size_t max_size);
  /* prepend prepends t onto s. max_size is the allocated size of s.
      Safely handles buffer bounds.
      */

void append(char* s, const char* t, size_t max_size);
  /* append appends t onto s. max_size is the allocated size of s.
      Safely handles buffer bounds.
      */

// Date/Time functions
void custom_date_formatted(void);
  /* custom_date_formatted prints detailed date and time information
      at time of execution.
      */

char* number_suffix(int x);
  /* number_suffix returns standard numeral suffix (st, nd, rd, th)
      for x. Ex.: x = 1, returns "st".
      Caller is responsible for freeing the returned string.
      */

// Conversion functions
int char_to_int(const char* s);
  /* char_to_int converts and returns s as an integer. Assumes s is an
      integer in character string format.
      */

char* int_to_char(int x);
  /* int_to_char converts and returns x as a character string.
      Caller is responsible for freeing the returned string.
      */

// Input validation
int validate_menu_input(const char* input, int min, int max);
  /* validate_menu_input validates that input is an integer within range.
      Returns: the validated integer, or -1 on error
      */

// Logging functions
void log_message(const char* message);
  /* log_message writes a timestamped message to the log file.
      */

void log_command(const char* command, int exit_code);
  /* log_command logs a system command and its exit status.
      */

#endif // UTILITY_FUNCTIONS_H

/* This program demystifies update process
   Author:	Vincent T. Mossman
   Compile by:  gcc -o system_update system_update.c
   Run by:	./system_update [-option]
*/

#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

// helper functions
void tell_user(char* message);
  /* tell_user sends message to the console for user to read. Default
      formatting - two newlines before message, one newline following.
      Note: Maximum message length is 99 characters.
      */
void tell_system(char* command);
  /* tell_system sends command to system to be executed.
      */
void prepend(char* s, const char* t);
  /* prepend prepends t onto s. Assumes s has enough space allocated
      for combined string.
      */
void append(char* s, const char* t);
  /* append appends t onto s. Assumes s has enough space allocated
      for combined string.
      */
void custom_date_formatted();
  /* custom_date_formatted prints detailed date and time information
      at time of execution.
      */
char* number_suffix(int x);
  /* number_suffix returns standard numeral suffix (st, nd, rd, th)
      for x. Ex.: x = 1, returns "st". Note: Current version effective
      only below 111.
      */
int char_to_int(char* s);
  /* char_to_int converts and returns s as an integer. Assumes s is an
      integer in character string format.
      */
char* int_to_char(int x);
  /* int_to_char converts and returns x as a character.
      */

void complete_details() {

  tell_user("==== UPDATE COMPLETE ====");
  tell_system("apt -v");
  custom_date_formatted();
  tell_user("A reboot is recommended.");

} // end complete_detils

void force_update() {

  /* 1.) Download package information from all configured sources.
     2.) Install available upgrades of all packages currently installed
          on the system. Remove currently installed packages if this is
          needed to upgrade the system as a whole.
     3.) Remove packages that were automatically installed to satisfy
          dependencies for other packages and are now no longer needed
          because dependencies changed or the package(s) needing them
          were removed.
     4.) Reboot the machine.
      */

  // 1.)
  tell_user("==== UPDATE PACKAGE INFORMATION ====");
  tell_system("sudo apt update");

  // 2.)
  tell_user("==== INSTALL ALL AVAILABLE PACKAGE UPGRADES ====");
  tell_system("sudo apt full-upgrade");

  // 3.)
  tell_user("==== REMOVE OBSOLETE PACKAGES ====");
  tell_system("sudo apt autoremove");

  // 4.)
  tell_user("==== SCHEDULE REBOOT ====");
  tell_system("sudo shutdown -r");

} // end force_update

void all_update() {

  /* 1.) Download package information from all configured sources.
     2.) Install available upgrades of all packages currently installed
          on the system. Remove currently installed packages if this is
          needed to upgrade the system as a whole.
     3.) Remove packages that were automatically installed to satisfy
          dependencies for other packages and are now no longer needed
          because dependencies changed or the package(s) needing them
          were removed.
     4.) Display update details.
      */

  // 1.)
  tell_user("==== UPDATE PACKAGE INFORMATION ====");
  tell_system("sudo apt update");

  // 2.)
  tell_user("==== INSTALL ALL AVAILABLE PACKAGE UPGRADES ====");
  tell_system("sudo apt full-upgrade");

  // 3.)
  tell_user("==== REMOVE OBSOLETE PACKAGES ====");
  tell_system("sudo apt autoremove");

  // 4.)
  complete_details();

} // end all_update

void critical_update() {

  /* 1.) Download package information from all configured sources.
     2.) Install available upgrades of all packages currently installed
          on the system.
     3.) Display update details.
      */

  // 1.)
  tell_user("==== UPDATE PACKAGE INFORMATION ====");
  tell_system("sudo apt update");

  // 2.)
  tell_user("==== INSTALL ALL AVAILABLE PACKAGE UPGRADES ====");
  tell_system("sudo apt upgrade");

  // 3.)
  complete_details();

} //end critical_update

int main(int argc, char* argv[]) {

  if (argc > 2) {

    tell_user("Invalid command");

  } // end if

  if (argc == 1) {

    tell_user("CODE WORK AHEAD");
    critical_update();
    // main_menu(); **UNDER CONSTRUCTION**

  } // end if

} // end main



void tell_user(char* message) {

  char* deep_message = malloc(120);
  strcpy(deep_message, message);
  prepend(deep_message, "echo \\\\n\\\\n");
  append(deep_message, "\\\\n");

  system(deep_message);

} // end tell_user

void tell_system(char* command) {

  system(command);

} // end tell_system

void prepend(char* s, const char* t) {

  size_t len_t = strlen(t);
  size_t len_s = strlen(s);
  size_t i;

  memmove(s + len_t, s, len_s + 1);

  for (i = 0; i < len_t; ++i) {

    s[i] = t[i];

  } // end for

} // end prepend

void append(char* s, const char* t) {

  size_t len_t = strlen(t);
  size_t len_s = strlen(s);
  size_t i;

  for (i = 0; i < len_t; ++i) {

    s[len_s + i] = t[i];

  } // end for

  s[len_s + len_t + 1] = '\0';

} // end append

void custom_date_formatted() {

  char* date_line_one = malloc(100);

  time_t t = time(NULL);
  struct tm timeinfo = *localtime(&t);

  int int_day = timeinfo.tm_mday;
  char* char_day = int_to_char(int_day);
  char* suffix = number_suffix(int_day);
  strcpy(date_line_one, "date '+%-I:%M%P on %A, the ");
  append(date_line_one, char_day);
  append(date_line_one, suffix);
  append(date_line_one, " day of %B, %Y'");

  tell_system(date_line_one);
  tell_system("date '+At precisely %-S seconds and %-N nanoseconds'");

} // end custom_date_formatted

char* number_suffix(int x) {

  char* suffix = malloc(3);

  if (x % 10 == 1 && x != 11) {

    strcpy(suffix, "st");

  } else if (x % 10 == 2 && x != 12) {

    strcpy(suffix, "nd");

  } else if (x % 10 == 3 && x != 13) {

    strcpy(suffix, "rd");

  } else {

    strcpy(suffix, "th");

  } // end if

  return suffix;

} // end number_suffix

int char_to_int(char* s) {

  int x;

  sscanf(s, "%d", &x);

  return x;

}  // end char_to_int

char* int_to_char(int x) {

  int digits = 1;
  int y = x;
  int builder;

  for (int i = x; i >= 10; i /= 10) {

    digits++;

  } // end for

  char* convert = malloc(digits + 1);
  convert[digits] = '\0';

  for (int i = digits - 1; i >= 0; --i) {

    builder = y % 10;

    switch (builder) {

      case 1 :
        convert[i] = '1';
        break;
      case 2 :
        convert[i] = '2';
        break;
      case 3 :
        convert[i] = '3';
        break;
      case 4 :
        convert[i] = '4';
        break;
      case 5 :
        convert[i] = '5';
        break;
      case 6 :
        convert[i] = '6';
        break;
      case 7 :
        convert[i] = '7';
        break;
      case 8 :
        convert[i] = '8';
        break;
      case 9 :
        convert[i] = '9';
        break;
      default :
        convert[i] = '0';

    } // end switch

    y /= 10;

  } // end for

  return convert;

} // end int_to_char

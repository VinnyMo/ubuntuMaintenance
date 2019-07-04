/* This program demystifies update process
   Author:	Vincent T. Mossman
   Compile by:  gcc -o system_update system_update.c
   Run by:	./system_update [-option]
*/

#include "utility_functions.c"
  /* utility_functions.c contains custom functions that have use in
      this program, as well as others. Many standard libraries are
      also #included in this file.
      */

// additional functions
void main_menu();
  /* main_menu displays a formatted menu as a default function of the
      program. Dynamically takes user input and displays requested
      information.
      */
void execute(char* option);
  /* execute executes update option based on user command.
     */

void complete_details() {

  tell_user("==== UPDATE COMPLETE ====");
  tell_system("apt -v");
  custom_date_formatted();
  tell_user("A reboot is recommended.");

} // end complete_detils

void force_update() {

  /* 1.a.) Download package information from all configured sources.
     1.b.) Install available upgrades of all packages currently installed
          on the system. Remove currently installed packages if this is
          needed to upgrade the system as a whole.
     3.) Remove packages that were automatically installed to satisfy
          dependencies for other packages and are now no longer needed
          because dependencies changed or the package(s) needing them
          were removed.
     4.) Reboot the machine.
      */

  // 1.)
  tell_user("==== UPDATE PACKAGE INFORMATION AND INSTALL ALL AVAILABLE PACKAGE UPGRADES ====");
  tell_system("sudo apt update && sudo apt full-upgrade");

  // 2.)
  tell_user("==== REMOVE OBSOLETE PACKAGES ====");
  tell_system("sudo apt autoremove -y");

  // 3.)
  tell_user("==== SCHEDULE REBOOT ====");
  tell_system("sudo shutdown -r");
  tell_user_custom_formatting("Current system time:", 1, 0);
  custom_date_formatted();
  tell_user("Logout or expect session disconnect");

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

  if (argc == 1) {

    main_menu();

  } else if (argc == 2) {

    execute(argv[1]);

  } else {

    tell_user("Invalid command");

  } // end if

} // end main

void main_menu() {

  tell_system("clear");
  tell_user_custom_formatting("==== SYSTEM UPDATE ====", 1, 1);
  tell_user_custom_formatting("1.\\) Force Update", 0, 0);
  tell_user_custom_formatting("2.\\) All Update", 0, 0);
  tell_user_custom_formatting("3.\\) Critical Update", 0, 0);
  tell_user_custom_formatting("4.\\) Information", 0, 1);
  tell_user_no_formatting("\\>");

  char menuChoice;
  scanf("%s", &menuChoice);

  tell_user(menuChoice);

} // end main_menu

void execute(char* option) {

  if (option[1] == 'f') {

    force_update();

  } else if (option[1] == 'a') {

    all_update();

  } else if (option[1] == 'c') {

    critical_update();

  } else {

    tell_user("Invalid command");

  } // end if

} // end execute

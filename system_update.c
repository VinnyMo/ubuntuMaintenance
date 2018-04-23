/* This program demystifies update process
   Author:	Vincent T. Mossman
   Compile by:  gcc -o system_update system_update.c
   Run by:	./system_update [-option]
*/

#include "utility_functions.c"

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

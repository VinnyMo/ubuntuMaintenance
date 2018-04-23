/* A collection of useful functions. */

// content summary
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

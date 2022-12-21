#include "../check.h"
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  int usr_local = open("/usr/local/bin", O_RDONLY, O_DIRECTORY);
  if (usr_local == -1) perror("usr_local");
  assert(usr_local != -1);
  // int yay = openat(usr_local, "./yay", O_RDONLY, 0);
  // perror("yay");
  // assert(yay != -1);
  check_access("/usr/local/bin/../hey");
  int hey = openat(usr_local, "../hey", O_RDONLY, 0);
  if (hey == -1) perror("hey");
  assert(hey != -1);
  return 0;
}

#include "../check.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
  if (argc != 2) {
    return 1;
  }
  char *mode = argv[1];
  if (strcmp(mode, "simple") == 0) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt1/goodbye.txt");
  } else if (strcmp(mode, "overlap1") == 0) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt0/mnt1/goodbye.txt");
    check_dir_entry_size("/mnt0", 1);
    check_dir_entry_size("/mnt0/mnt1", 1);
  } else if (strcmp(mode, "overlap2") == 0) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt0/mnt1/goodbye.txt");
  } else {
    fprintf(stderr, "Unknown mode: %s\n", mode);
    return 1;
  }
  return 0;
}

#include "../check.h"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    return 1;
  }
  char *mode = argv[1];
  if (strcmp(mode, "phase1") == 0) {
    check_file_exists("/mnt0/hello.txt");
  } else if (strcmp(mode, "phase2") == 0) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt1/goodbye.txt");
  } else if (strcmp(mode, "phase3") == 0) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt1/x.txt");
    check_file_not_exists("/mnt1/goodbye.txt");
  } else {
    fprintf(stderr, "Unknown mode: %s\n", mode);
    return 1;
  }
  return 0;
}

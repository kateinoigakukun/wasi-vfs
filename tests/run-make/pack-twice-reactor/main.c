#include "../check.h"
#include <stdio.h>

#pragma clang diagnostic ignored "-Wunknown-attributes"
__attribute__((export_name("check")))
int check(int stage) {
  if (stage == 0) {
    check_file_exists("/mnt0/hello.txt");
  } else if (stage == 1) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt1/goodbye.txt");
  } else if (stage == 2) {
    check_file_exists("/mnt0/hello.txt");
    check_file_exists("/mnt1/x.txt");
    check_file_not_exists("/mnt1/goodbye.txt");
  } else {
    fprintf(stderr, "Unknown stage: %d\n", stage);
    return 1;
  }
  return 0;
}

int main(int argc, char *argv[]) {
  return 1;
}

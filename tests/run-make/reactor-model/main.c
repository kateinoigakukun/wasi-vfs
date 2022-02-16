#include "../check.h"
#include <stdio.h>

#pragma clang diagnostic ignored "-Wunknown-attributes"
__attribute__((export_name("check")))
void check(void) {
  printf("checking\n");
  check_file_exists("/hello.txt");
  check_dir_entry_size("/", 1);
}

int main(int argc, char *argv[]) {
  return 1;
}

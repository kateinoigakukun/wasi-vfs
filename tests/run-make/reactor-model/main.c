#include "../check.h"
#include <stdio.h>

#pragma clang diagnostic ignored "-Wunknown-attributes"
__attribute__((export_name("check")))
void check(void) {
  check_file_exists("/foo.txt");
  check_dir_entry_size("/", 1);
  check_file_exists("/run/bar.txt");
  check_dir_entry_size("/run", 1);
}

int main(int argc, char *argv[]) {
  return 1;
}

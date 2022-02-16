#include "../check.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
  check_file_exists("/hello.txt");
  check_dir_entry_size("/", 1);
  return 0;
}

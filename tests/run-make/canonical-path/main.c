#include "../check.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
  check_file_exists("/mnt/a/b/c.txt");
  check_file_exists("/mnt/a/./b/c.txt");
  check_file_exists("/mnt/a/././b/c.txt");
  check_file_exists("/mnt/a//b/c.txt");
  return 0;
}

#include "../check.h"
#include <fcntl.h>
#include <errno.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  int root = open("/", O_RDONLY, O_DIRECTORY);
  assert(errno == ENOTCAPABLE);
  assert(root == -1);
  return 0;
}

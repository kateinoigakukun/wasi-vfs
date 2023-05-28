#include "../check.h"
#include <fcntl.h>
#include <errno.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  int root = open("/", O_RDONLY, O_DIRECTORY);
  // wasi-libc returns ENOENT since wasi-sdk 19, but returns ENOTCAPABLE before that
  // https://github.com/WebAssembly/wasi-libc/pull/370
  assert(errno == ENOENT || errno == ENOTCAPABLE);
  assert(root == -1);
  return 0;
}

#include "../check.h"
#include <fcntl.h>
#include <errno.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  // Check that pagination of readdir is working
  // 200 entries is enough to fill the initial readdir buffer
  check_dir_entry_size("/mnt/grow-buffer", 200);
  return 0;
}

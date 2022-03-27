#include "../check.h"
#include <sys/stat.h>
#include <fcntl.h>
#include <wasi/api.h>
#include <errno.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  int err;
  int file1 = open("/fixtures/file1.txt", O_RDONLY);
  assert(file1 != -1);

  struct stat st;
  err = fstat(file1, &st);
  assert(err == 0);

  char buffer[st.st_size];

  __wasi_filesize_t off;

  // reset the cursor
  assert(__wasi_fd_seek(file1, 0, SEEK_SET, &off) == 0);
  assert(off == 0);

  // keep the initial cursor position
  assert(__wasi_fd_seek(file1, 0, SEEK_CUR, &off) == 0);
  assert(off == 0);
  assert(read(file1, buffer, st.st_size) == st.st_size);

  assert(__wasi_fd_seek(file1, 2, SEEK_SET, &off) == 0);
  assert(off == 2);
  assert(read(file1, buffer, st.st_size) == st.st_size - 2);

  assert(__wasi_fd_seek(file1, st.st_size, SEEK_SET, &off) == 0);
  assert(off == st.st_size);
  assert(read(file1, buffer, 1) == 0);

  assert(__wasi_fd_seek(file1, st.st_size + 1, SEEK_SET, &off) == 0);
  assert(err == 0);
  assert(read(file1, buffer, 1) == 0);

  // reset the cursor
  assert(__wasi_fd_seek(file1, 0, SEEK_SET, &off) == 0);

  assert(__wasi_fd_seek(file1, 0, SEEK_END, &off) == 0);
  assert(err == 0);
  assert(read(file1, buffer, 1) == 0);

  assert(__wasi_fd_seek(file1, -1, SEEK_END, &off) == 0);
  assert(err == 0);
  assert(read(file1, buffer, 1) == 1);

  assert(__wasi_fd_seek(file1, -st.st_size, SEEK_END, &off) == 0);
  assert(err == 0);
  assert(off == 0);
  assert(read(file1, buffer, st.st_size) == st.st_size);

  return 0;
}

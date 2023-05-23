#include <assert.h>
#include <stdio.h>
#include <time.h>
#include <fcntl.h>
#include <sys/ioctl.h>

int main(int argc, char *argv[]) {
  int ret = clock_nanosleep(CLOCK_MONOTONIC, 0, 0, 0);
  assert(ret == 0);

  // Check ioctl(fd, FIONREAD, ...)
  int fd = open("/dev/polling", O_RDONLY);
  assert(fd >= 0);
  int val = 0;
  ret = ioctl(fd, FIONREAD, &val);
  assert(ret == 0);
  assert(val == 6); // "hello\n"

  return 0;
}

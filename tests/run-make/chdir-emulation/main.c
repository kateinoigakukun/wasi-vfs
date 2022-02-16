#include <assert.h>
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include "../check.h"

void check_file_content(const char *path, const char *expected) {
  FILE *fp;
  char *line = NULL;
  size_t len = 0;
  ssize_t read;

  fp = fopen(path, "r");
  if (fp == NULL) {
    perror("fopen");
    exit(EXIT_FAILURE);
  }

  read = getline(&line, &len, fp);
  if(read == -1) {
    perror("getline");
    exit(EXIT_FAILURE);
  }
  if (strcmp(line, expected) != 0) {
    printf("expected '%s' but got '%s'\n", expected, line);
    fflush(stdout);
    exit(EXIT_FAILURE);
  }

  fclose(fp);
  if (line)
    free(line);
}

int main(int argc, char *argv[]) {
  // it's important to use chdir to enable __wasilibc_find_relpath_alloc path
  chdir("/mnt");

  check_file_content("hello.txt", "Hello\n");
  check_file_content("hello.txt", "Hello\n");
  return 0;
}

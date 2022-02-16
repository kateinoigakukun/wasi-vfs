#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
  if (strcmp(line, expected) != 0) {
    printf("expected '%s' but got '%s'\n", expected, line);
    fflush(stdout);
    exit(EXIT_FAILURE);
  }

  fclose(fp);
  if (line)
    free(line);
}

int main(void) {
  check_file_content("/hello.txt", "from host\n");
  check_file_content("/usr/hello.txt", "from vfs\n");
  check_file_content("/usr/subdir/inner.txt", "inner file\n");
  exit(EXIT_SUCCESS);
}

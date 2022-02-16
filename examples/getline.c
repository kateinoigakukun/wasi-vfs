#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
  char *line = NULL;
  size_t len = 0;
  ssize_t nread;
  FILE *f;

  if (argc != 2) {
    printf("No file specified\n");
    exit(EXIT_FAILURE);
  }

  if ((f = fopen(argv[1], "r")) == NULL) {
    perror("fopen");
    exit(EXIT_FAILURE);
  }
  while ((nread = getline(&line, &len, f)) != -1) {
    fwrite(line, nread, 1, stdout);
  }
  return 0;
}

#ifndef RUN_MAKE_CHECK_H
#define RUN_MAKE_CHECK_H

#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static inline void _check_file_exists(const char *path, const char *file,
                                      const char *func, int line) {
  if (!path) {
    fprintf(stderr, "check_file_exists: path is null in %s:%d (%s)\n", file,
            line, func);
    exit(1);
    return;
  }
  FILE *f = fopen(path, "r");
  if (!f) {
    fprintf(stderr, "File %s not found in %s:%d (%s)\n", path, file, line,
            func);
    exit(1);
  }
  fclose(f);
}

#define check_file_exists(path)                                                \
  _check_file_exists(path, __FILE__, __func__, __LINE__)

static inline void _check_file_not_exists(const char *path, const char *file,
                                          const char *func, int line) {
  if (!path) {
    fprintf(stderr, "check_file_not_exists: path is null in %s:%d (%s)\n", file,
            line, func);
    exit(1);
    return;
  }
  FILE *f = fopen(path, "r");
  if (f) {
    fprintf(stderr, "File %s unexpectedly found in %s:%d (%s)\n", path, file,
            line, func);
    fclose(f);
  }
}

#define check_file_not_exists(path)                                            \
  _check_file_not_exists(path, __FILE__, __func__, __LINE__)

static inline void _check_dir_entry_size(const char *dir, int expected,
                                         const char *file, const char *func,
                                         int line) {
  DIR *d = opendir(dir);
  if (!d) {
    fprintf(stderr, "Directory %s not found in %s:%d (%s)\n", dir, file, line,
            func);
    exit(1);
  }
  struct dirent *de;
  int count = 0;
  while ((de = readdir(d))) {
    if (strlen(de->d_name) == 1 && de->d_name[0] == '.')
      continue;
    if (strlen(de->d_name) == 2 && de->d_name[0] == '.' && de->d_name[1] == '.')
      continue;
    count++;
  }
  closedir(d);
  if (count != expected) {
    fprintf(stderr, "Directory %s should contain %d files, but contains %d\n",
            dir, expected, count);
    exit(1);
  }
}

#define check_dir_entry_size(dir, expected)                                    \
  _check_dir_entry_size(dir, expected, __FILE__, __func__, __LINE__)

#endif

static inline void _check_access(const char *path, const char *file,
                                 const char *func, int line) {
  if (!path) {
    fprintf(stderr, "check_access: path is null in %s:%d (%s)\n", file, line,
            func);
    exit(1);
    return;
  }
  int e = access(path, F_OK);
  if (e != 0) {
    fprintf(stderr, "File %s is not accessible %s:%d (%s)\n", path, file, line,
            func);
    perror("access");
    exit(1);
  }
}

#define check_access(path) _check_access(path, __FILE__, __func__, __LINE__)

static inline void _check_file_line(const char *path, const char *expected,
                                    const char *file, const char *func, int line) {
  FILE *fp;
  char *actual = NULL;
  size_t len = 0;
  ssize_t read;

  fp = fopen(path, "r");
  if (fp == NULL) {
    fprintf(stderr, "Failed to open %s %s:%d (%s)\n", path, file, line,
            func);
    perror("fopen");
    exit(1);
  }

  read = getline(&actual, &len, fp);
  if (strcmp(actual, expected) != 0) {
    fprintf(stderr, "expected '%s' but got '%s' in %s %s:%d (%s)\n", expected, actual, path, file, line, func);
    exit(1);
  }

  fclose(fp);
  if (actual)
    free(actual);
}

#define check_file_line(path, expected) _check_file_line(path, expected, __FILE__, __func__, __LINE__)

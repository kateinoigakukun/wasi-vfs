// This file is automatically generated, DO NOT EDIT
//
// To regenerate this file run the `crates/wasi-libc-trampoline-bindgen` command

#include <stdint.h>

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_advise(int32_t arg0, int64_t arg1, int64_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_fd_advise(int32_t arg0, int64_t arg1, int64_t arg2, int32_t arg3);
  return wasi_vfs_fd_advise(arg0, arg1, arg2, arg3);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_allocate(int32_t arg0, int64_t arg1, int64_t arg2) {
  extern int32_t wasi_vfs_fd_allocate(int32_t arg0, int64_t arg1, int64_t arg2);
  return wasi_vfs_fd_allocate(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_close(int32_t arg0) {
  extern int32_t wasi_vfs_fd_close(int32_t arg0);
  return wasi_vfs_fd_close(arg0);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_datasync(int32_t arg0) {
  extern int32_t wasi_vfs_fd_datasync(int32_t arg0);
  return wasi_vfs_fd_datasync(arg0);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_fdstat_get(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_fdstat_get(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_fdstat_get(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_fdstat_set_flags(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_fdstat_set_flags(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_fdstat_set_flags(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_fdstat_set_rights(int32_t arg0, int64_t arg1, int64_t arg2) {
  extern int32_t wasi_vfs_fd_fdstat_set_rights(int32_t arg0, int64_t arg1, int64_t arg2);
  return wasi_vfs_fd_fdstat_set_rights(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_filestat_get(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_filestat_get(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_filestat_get(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_filestat_set_size(int32_t arg0, int64_t arg1) {
  extern int32_t wasi_vfs_fd_filestat_set_size(int32_t arg0, int64_t arg1);
  return wasi_vfs_fd_filestat_set_size(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_filestat_set_times(int32_t arg0, int64_t arg1, int64_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_fd_filestat_set_times(int32_t arg0, int64_t arg1, int64_t arg2, int32_t arg3);
  return wasi_vfs_fd_filestat_set_times(arg0, arg1, arg2, arg3);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_pread(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4) {
  extern int32_t wasi_vfs_fd_pread(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4);
  return wasi_vfs_fd_pread(arg0, arg1, arg2, arg3, arg4);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_prestat_get(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_prestat_get(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_prestat_get(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_prestat_dir_name(int32_t arg0, int32_t arg1, int32_t arg2) {
  extern int32_t wasi_vfs_fd_prestat_dir_name(int32_t arg0, int32_t arg1, int32_t arg2);
  return wasi_vfs_fd_prestat_dir_name(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_pwrite(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4) {
  extern int32_t wasi_vfs_fd_pwrite(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4);
  return wasi_vfs_fd_pwrite(arg0, arg1, arg2, arg3, arg4);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_read(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_fd_read(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3);
  return wasi_vfs_fd_read(arg0, arg1, arg2, arg3);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_readdir(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4) {
  extern int32_t wasi_vfs_fd_readdir(int32_t arg0, int32_t arg1, int32_t arg2, int64_t arg3, int32_t arg4);
  return wasi_vfs_fd_readdir(arg0, arg1, arg2, arg3, arg4);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_renumber(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_renumber(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_renumber(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_seek(int32_t arg0, int64_t arg1, int32_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_fd_seek(int32_t arg0, int64_t arg1, int32_t arg2, int32_t arg3);
  return wasi_vfs_fd_seek(arg0, arg1, arg2, arg3);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_sync(int32_t arg0) {
  extern int32_t wasi_vfs_fd_sync(int32_t arg0);
  return wasi_vfs_fd_sync(arg0);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_tell(int32_t arg0, int32_t arg1) {
  extern int32_t wasi_vfs_fd_tell(int32_t arg0, int32_t arg1);
  return wasi_vfs_fd_tell(arg0, arg1);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_fd_write(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_fd_write(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3);
  return wasi_vfs_fd_write(arg0, arg1, arg2, arg3);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_create_directory(int32_t arg0, int32_t arg1, int32_t arg2) {
  extern int32_t wasi_vfs_path_create_directory(int32_t arg0, int32_t arg1, int32_t arg2);
  return wasi_vfs_path_create_directory(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_filestat_get(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4) {
  extern int32_t wasi_vfs_path_filestat_get(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4);
  return wasi_vfs_path_filestat_get(arg0, arg1, arg2, arg3, arg4);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_filestat_set_times(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int64_t arg4, int64_t arg5, int32_t arg6) {
  extern int32_t wasi_vfs_path_filestat_set_times(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int64_t arg4, int64_t arg5, int32_t arg6);
  return wasi_vfs_path_filestat_set_times(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_link(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5, int32_t arg6) {
  extern int32_t wasi_vfs_path_link(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5, int32_t arg6);
  return wasi_vfs_path_link(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_open(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int64_t arg5, int64_t arg6, int32_t arg7, int32_t arg8) {
  extern int32_t wasi_vfs_path_open(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int64_t arg5, int64_t arg6, int32_t arg7, int32_t arg8);
  return wasi_vfs_path_open(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_readlink(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5) {
  extern int32_t wasi_vfs_path_readlink(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5);
  return wasi_vfs_path_readlink(arg0, arg1, arg2, arg3, arg4, arg5);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_remove_directory(int32_t arg0, int32_t arg1, int32_t arg2) {
  extern int32_t wasi_vfs_path_remove_directory(int32_t arg0, int32_t arg1, int32_t arg2);
  return wasi_vfs_path_remove_directory(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_rename(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5) {
  extern int32_t wasi_vfs_path_rename(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4, int32_t arg5);
  return wasi_vfs_path_rename(arg0, arg1, arg2, arg3, arg4, arg5);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_symlink(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4) {
  extern int32_t wasi_vfs_path_symlink(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3, int32_t arg4);
  return wasi_vfs_path_symlink(arg0, arg1, arg2, arg3, arg4);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_path_unlink_file(int32_t arg0, int32_t arg1, int32_t arg2) {
  extern int32_t wasi_vfs_path_unlink_file(int32_t arg0, int32_t arg1, int32_t arg2);
  return wasi_vfs_path_unlink_file(arg0, arg1, arg2);
}

__attribute__((weak))
int32_t __imported_wasi_snapshot_preview1_poll_oneoff(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3) {
  extern int32_t wasi_vfs_poll_oneoff(int32_t arg0, int32_t arg1, int32_t arg2, int32_t arg3);
  return wasi_vfs_poll_oneoff(arg0, arg1, arg2, arg3);
}



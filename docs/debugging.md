# Debugging Tips

## Trace WASI system calls

wasi-vfs provides a `strace`-like system call tracer for some WASI system calls where it hooks.
The feature is hidden by default, so if you want to enable this, please build `libwasi_vfs.a` by the following command:

```console
$ cargo build --target wasm32-unknown-unknown --features trace-syscall
```

And also, you need to set environment variable `WASI_VFS_TRACE=1` when running your WASI application to print the tracing info.

```strace
$ wasmtime run --env WASI_VFS_TRACE=1 .tmp/main.packed.wasm
fd_prestat_get(fd: 3)
fd_prestat_dir_name(fd: 3, path: 92928, path_len: 1)
fd_prestat_get(fd: 4)
fd_prestat_dir_name(fd: 4, path: 92928, path_len: 4)
fd_prestat_get(fd: 5)
fd_prestat_get returns 8
fd_fdstat_get(fd: 3)
path_open(fd: 3, dirflags: 1, path: hello.txt, oflags: 0, fs_rights_base: 263716542, fs_rights_inheriting: 267911167, fdflags: 0)
fd_read(fd: 5, iovs: 85176, iovs_len: 1)
fd_close(fd: 5)
fd_fdstat_get(fd: 4)
path_open(fd: 4, dirflags: 1, path: hello.txt, oflags: 0, fs_rights_base: 2121858, fs_rights_inheriting: 2121858, fdflags: 0)
fd_read(fd: 6, iovs: 85176, iovs_len: 1)
fd_close(fd: 6)
fd_fdstat_get(fd: 4)
path_open(fd: 4, dirflags: 1, path: subdir/inner.txt, oflags: 0, fs_rights_base: 2121858, fs_rights_inheriting: 2121858, fdflags: 0)
fd_read(fd: 7, iovs: 85176, iovs_len: 1)
fd_close(fd: 7)
```

// A dummy symbol to force the linker to include this file
// This symbol is used in lib.rs.
void __wasi_vfs_force_link_init(void) {}

#pragma clang diagnostic ignored "-Wunknown-attributes"
__attribute__((export_name("wasi_vfs_pack_fs")))
void export_wasi_vfs_pack_fs(void) {
  extern void __internal_wasi_vfs_pack_fs(void);
  __internal_wasi_vfs_pack_fs();
}

// wasi-libc reserves 50~100 constructor, and __wasilibc_populate_preopens calls
// fs syscall, so this need to be done before that.
__attribute__((constructor(40)))
void __wasi_vfs_rt_init(void) {
  extern void __internal_wasi_vfs_rt_init(void);
  __internal_wasi_vfs_rt_init();
}

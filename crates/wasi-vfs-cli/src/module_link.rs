/// Given these two modules:
///
/// libwasi_vfs.wasm
/// ```webassembly
/// (module
///     (import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))
///     (export "wasi_vfs_fd_read" (func $wasi_vfs_fd_read.command_export))
/// )
/// ```
///
/// main.wasm
/// ```webassembly
/// (module
///     (import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))
/// )
/// ```
///
/// This function generates the following adapter module:
///
/// ```webassembly
/// (adapter module
///     (import "wasi_snapshot_preview1" (instance $wasi_snapshot_preview1
///         (export "fd_read" (func (param i32 i32 i32 i32) (result i32)))
///     ))
///
///     ;; libwasi_vfs.wasm
///     (module $Vfs
///         (import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))
///         (export "wasi_vfs_fd_read" (func $wasi_vfs_fd_read.command_export))
///     )
///
///     ;; main.wasm
///     (module $Main
///         (import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))
///     )
///
///     (instance $VfsInstance (instantiate $Vfs
///         (import "wasi_snapshot_preview1" (instance $wasi_snapshot_preview1))
///     ))
///
///     (instance $MainInstance (instantiate $Main
///         (import "wasi_snapshot_preview1" (instance $VfsInstance))
///     ))
/// )
/// ```
///
pub fn link(_main_bytes: &[u8]) {
    unimplemented!(
        r#"
    This feature requires module-linking and interface-type proposals, and interface-type support of WASI.
    Current WASI interface (wasi_snapshot_preview1) requires the callee to access the caller's linear memory,
    so VFS instance has to import it while exporting WASI APIs. Also, calling real WASI API from VFS instance
    requires the VFS instance to pass the pointer of the main instance memory. In theory, a virtualized WASI API
    in VFS only calls corresponding real WASI API, so it's possible to implement this feature without interface-type
    by annotating which parameters and results are pull/push address, and who has the ownership of the pointer.
    However, it's hard to maintain such annotations, so I decided to wait the interface-type proposal and WASI adaptation
    of shared-nothing architecture.
    "#
    )
}

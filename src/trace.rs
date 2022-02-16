#[cfg(feature = "trace-syscall")]
fn is_tracing_enabled() -> bool {
    extern "C" {
        fn getenv(name: *const i8) -> *const i8;
    }
    let wasi_vfs_trace = std::ffi::CString::new("WASI_VFS_TRACE").unwrap();
    unsafe { !getenv(wasi_vfs_trace.as_ptr()).is_null() }
}

#[cfg(feature = "trace-syscall")]
pub(crate) fn trace_syscall_entry(args: std::fmt::Arguments<'_>) {
    if !is_tracing_enabled() {
        return;
    }
    let message = std::fmt::format(args);
    let data = [wasi::Ciovec {
        buf: message.as_ptr(),
        buf_len: message.len(),
    }];
    let stderr = 2;
    unsafe {
        wasi::fd_write(stderr, &data).unwrap();
    }
}

#[cfg(feature = "trace-syscall")]
pub(crate) fn trace_syscall_error(name: &str, errno: crate::Error) {
    if !is_tracing_enabled() {
        return;
    }
    let message = format!("{} returns {}\n", name, errno.raw());
    let data = [wasi::Ciovec {
        buf: message.as_ptr(),
        buf_len: message.len(),
    }];
    let stderr = 2;
    unsafe {
        wasi::fd_write(stderr, &data).unwrap();
    }
}

pub(crate) fn print(message: String) {
    let data = [wasi::Ciovec {
        buf: message.as_ptr(),
        buf_len: message.len(),
    }];
    let stdout = 1;
    unsafe {
        wasi::fd_write(stdout, &data).unwrap();
    }
}

//! wasi-vfs is a virtual file system compatible with WASI.

mod alloc;
mod embed;
mod trace;
mod trampoline_generated;
#[allow(unused_variables)]
mod wasi_snapshot_preview1;

use embed::LinkedStorage as DefaultStorage;
use embed::{EmbeddedFs, NodeIdTrait, Storage};

use std::{collections::HashMap, ffi::CStr};
use wasi::Fd;

/// User-facing file descriptor managed by wasi-vfs
pub(crate) type UserFd = u32;
/// Internal file descriptor managed by virtual file system backend
pub(crate) type Vfd = u32;

/// Generic internal file descriptor
#[derive(Clone, Copy, Debug)]
pub(crate) enum BackingFd {
    /// File descriptor managed by virtual file system backend
    Virtual(Vfd),
    /// File descriptor managed by a real WASI implementation
    Wasi(Fd),
}

/// Map of user-facing file descriptors to internal file descriptors.
type FdMap = HashMap<UserFd, BackingFd>;

struct FileSystem<S: Storage> {
    embedded_fs: EmbeddedFs<S>,
    fd_map: FdMap,
    next_fd: u32,
}

impl<S: Storage> FileSystem<S> {
    fn create(embedded_fs: EmbeddedFs<S>, preopened_vfds: &[Vfd]) -> Self {
        let mut fs = FileSystem {
            embedded_fs,
            fd_map: FdMap::new(),
            next_fd: 3,
        };
        // reserve stdin/stdout/stderr
        for fd in 0..=2 {
            fs.set_user_fd_at(BackingFd::Wasi(fd), fd);
        }

        for fd in 3.. {
            unsafe {
                match wasi::fd_prestat_get(fd) {
                    Ok(_) => (),
                    Err(wasi::ERRNO_BADF) => break,
                    Err(other) => {
                        panic!("failed to get prestat: {}", other);
                    }
                }
            }

            fs.issue_user_fd(BackingFd::Wasi(fd));
        }
        for vfd in preopened_vfds {
            let fd = fs.next_fd;
            fs.next_fd += 1;
            fs.fd_map.insert(fd, BackingFd::Virtual(*vfd));
        }
        fs
    }

    fn set_user_fd_at(&mut self, backing_fd: BackingFd, fd: UserFd) {
        self.fd_map.insert(fd, backing_fd);
    }
    fn issue_user_fd(&mut self, backing_fd: BackingFd) -> UserFd {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.fd_map.insert(fd, backing_fd);
        fd as UserFd
    }

    fn get_backing_fd(&self, user_fd: UserFd) -> Result<BackingFd, Error> {
        match self.fd_map.get(&user_fd) {
            Some(backing_fd) => Ok(*backing_fd),
            None => Err(wasi::ERRNO_BADF.into()),
        }
    }
}

struct GlobalState<S: Storage> {
    embedded_fs: Option<(EmbeddedFs<S>, Vec<Vfd>)>,
    overlay_fs: Option<FileSystem<S>>,
}

static mut GLOBAL_STATE: GlobalState<DefaultStorage> = GlobalState {
    embedded_fs: None,
    overlay_fs: None,
};

// `__internal_wasi_vfs_rt_init` is processed before the wasi-libc's initialization, which
// loads envirnoment variables and preopened directories. `getenv` is not available at this
// time, so use self-made `env_var` instead.
fn env_var(name: &str) -> Option<String> {
    let (count, buffer_size) = unsafe { wasi::environ_sizes_get().ok()? };
    if count == 0 {
        return None;
    }
    let mut offsets: Vec<*mut u8> = vec![std::ptr::null_mut(); count];
    let mut buffer: Vec<u8> = vec![0; buffer_size];
    unsafe {
        wasi::environ_get(offsets.as_mut_ptr(), buffer.as_mut_ptr()).ok()?;
    };
    for offset in offsets {
        let c_str = unsafe { CStr::from_ptr(offset as *const i8) };
        let pair = c_str.to_string_lossy();
        let mut pair = pair.splitn(2, '=');
        let key = pair.next()?;
        let value = pair.next()?;
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

fn get_or_create_overlay_fs() -> Option<&'static mut FileSystem<DefaultStorage>> {
    if env_var("__WASI_VFS_PACKING").is_some() {
        return None;
    }
    unsafe {
        if let Some(fs) = &mut GLOBAL_STATE.overlay_fs {
            Some(fs)
        } else {
            let (embedded_fs, preopened_vfds) = GLOBAL_STATE.embedded_fs.take()?;
            let fs = FileSystem::create(embedded_fs, &preopened_vfds);
            GLOBAL_STATE.overlay_fs = Some(fs);
            GLOBAL_STATE.overlay_fs.as_mut()
        }
    }
}

/// Packing-time entry point to scan the host file system.
#[no_mangle]
unsafe extern "C" fn __internal_wasi_vfs_pack_fs() {
    extern "C" {
        fn __wasi_vfs_force_link_init();
    }
    __wasi_vfs_force_link_init();

    std::panic::set_hook(Box::new(|info| {
        trace::print(format!("{}\n", info));
    }));
    let (mut fs, preopened_vfds) = if let Some((fs, vfds)) = GLOBAL_STATE.embedded_fs.take() {
        (fs, vfds)
    } else {
        (EmbeddedFs::default(), vec![])
    };

    let (preopened_vfds, prestats) =
        FsPacker::scan_preopened_dirs(&mut fs, preopened_vfds).unwrap();
    let packer = FsPacker::new(fs, preopened_vfds).unwrap();
    let fs = packer.pack(prestats).unwrap();
    GLOBAL_STATE.embedded_fs = Some(fs);

    #[cfg(not(feature = "module-linking"))]
    {
        extern "C" {
            fn __wasilibc_deinitialize_environ();
        }
        __wasilibc_deinitialize_environ();
    }
}

struct Prestat<S: Storage> {
    real_fd: u32,
    node_id: S::NodeId,
    link_id: S::LinkId,
}

struct FsPacker<S: Storage> {
    fs: EmbeddedFs<S>,
    preopened_vfds: Vec<Vfd>,
    verbose: bool,
}

trait DirVisitor<S: Storage> {
    fn visit_file(
        &mut self,
        path: &str,
        fd: u32,
        preopened_id: (S::NodeId, S::LinkId),
    ) -> Result<(), u16>;

    fn visit_dir(
        &mut self,
        prefix: &str,
        fd: u32,
        preopened_id: (S::NodeId, S::LinkId),
    ) -> Result<(), u16>;
}

fn walk_dir<S: Storage, V: DirVisitor<S>>(
    visitor: &mut V,
    prefix: &str,
    fd: u32,
    preopened_id: (S::NodeId, S::LinkId),
) -> Result<(), u16> {
    const DIRENT_DEFAULT_BUFFER_SIZE: usize = 4096;
    let mut offset = 0;
    let mut capacity = 0;
    let mut cookie = wasi::DIRCOOKIE_START;
    let mut buffer = vec![0; DIRENT_DEFAULT_BUFFER_SIZE];
    loop {
        if offset == capacity {
            capacity = unsafe { wasi::fd_readdir(fd, buffer.as_mut_ptr(), buffer.len(), cookie) }
                .expect("failed to readdir");
            offset = 0;
            if capacity == 0 {
                break;
            }
        }
        let data = &buffer[offset..capacity];
        let dirent_size = core::mem::size_of::<wasi::Dirent>();

        // when dirent is truncated, re-read it
        if data.len() < dirent_size {
            offset = capacity;
            continue;
        }

        let (dirent, data) = data.split_at(dirent_size);
        let dirent = unsafe { core::ptr::read_unaligned(dirent.as_ptr() as *const wasi::Dirent) };

        // when entry name is truncated
        if data.len() < dirent.d_namlen as usize {
            // when the buffer is not enough to read an big entry, realloc the buffer and re-read the entry
            if offset == 0 {
                let amt_to_add = buffer.capacity();
                buffer.extend(core::iter::repeat(0).take(amt_to_add));
            }
            offset = capacity;
            continue;
        }
        cookie = dirent.d_next;
        offset += dirent_size + dirent.d_namlen as usize;
        let name = &data[..dirent.d_namlen as usize];
        if name == b"." || name == b".." {
            continue;
        }

        let name = String::from_utf8(name.to_vec()).unwrap();
        let path = format!("{}/{}", prefix, name);
        let rights = wasi::RIGHTS_FD_READ
            | wasi::RIGHTS_FD_READDIR
            | wasi::RIGHTS_FD_FILESTAT_GET
            | wasi::RIGHTS_PATH_OPEN;

        match dirent.d_type {
            wasi::FILETYPE_DIRECTORY => {
                let oflags = wasi::OFLAGS_DIRECTORY;
                let child_fd = unsafe {
                    wasi::path_open(
                        fd,
                        wasi::LOOKUPFLAGS_SYMLINK_FOLLOW,
                        &name,
                        oflags,
                        rights,
                        rights,
                        0,
                    )
                }
                .map_err(|e| e.raw())
                .unwrap();

                visitor.visit_dir(&path, child_fd, preopened_id)?;
                walk_dir(visitor, &path, child_fd, preopened_id)?;

                unsafe {
                    wasi::fd_close(child_fd).expect("failed to close fd");
                }
            }
            wasi::FILETYPE_REGULAR_FILE => {
                let oflags = 0;
                let child_fd = unsafe {
                    wasi::path_open(
                        fd,
                        wasi::LOOKUPFLAGS_SYMLINK_FOLLOW,
                        &name,
                        oflags,
                        rights,
                        rights,
                        0,
                    )
                }
                .map_err(|e| e.raw())
                .unwrap();
                visitor.visit_file(&path, child_fd, preopened_id)?;
                unsafe {
                    wasi::fd_close(child_fd).expect("failed to close fd");
                }
            }
            _ => {}
        }
    }
    Ok(())
}

impl<S: Storage> FsPacker<S> {
    fn scan_preopened_dirs(
        fs: &mut EmbeddedFs<S>,
        mut preopened_vfds: Vec<Vfd>,
    ) -> Result<(Vec<Vfd>, Vec<Prestat<S>>), u16> {
        let mut preopened_dirs = Vec::new();
        'scan: for fd in 3.. {
            let stat = match unsafe { wasi::fd_prestat_get(fd) } {
                Ok(stat) => stat,
                Err(wasi::ERRNO_BADF) => break 'scan,
                Err(other) => {
                    return Err(other.raw());
                }
            };
            if stat.tag == wasi::PREOPENTYPE_DIR.raw() {
                preopened_dirs.push((fd, stat));
            }
        }

        let mut prestats = Vec::new();
        for (real_fd, stat) in preopened_dirs {
            let dir = unsafe {
                let mut prefix = vec![0; stat.u.dir.pr_name_len + 1];
                wasi::fd_prestat_dir_name(real_fd, prefix.as_mut_ptr(), stat.u.dir.pr_name_len)
                    .map_err(|e| e.raw())
                    .expect("failed to get dir name");
                let dir = CStr::from_bytes_with_nul(&prefix).unwrap();
                dir.to_string_lossy().to_string()
            };
            let (vfd, node_id, link_id) = fs.preopen_dir(dir);
            prestats.push(Prestat {
                real_fd,
                node_id,
                link_id,
            });
            preopened_vfds.push(vfd);
        }
        Ok((preopened_vfds, prestats))
    }

    fn new(fs: EmbeddedFs<S>, preopened_vfds: Vec<Vfd>) -> Result<Self, u16> {
        Ok(FsPacker {
            fs,
            preopened_vfds,
            verbose: env_var("WASI_VFS_VERBOSE")
                .map(|v| v == "1")
                .unwrap_or(false),
        })
    }

    fn pack(mut self, prestats: Vec<Prestat<S>>) -> Result<(EmbeddedFs<S>, Vec<Vfd>), u16> {
        for stat in prestats {
            walk_dir(&mut self, "", stat.real_fd, (stat.node_id, stat.link_id))?;
        }
        Ok((self.fs, self.preopened_vfds))
    }
}

impl<S: Storage> DirVisitor<S> for FsPacker<S> {
    fn visit_dir(
        &mut self,
        path: &str,
        _fd: u32,
        preopened_id: (S::NodeId, S::LinkId),
    ) -> Result<(), u16> {
        self.fs
            .create_dir(preopened_id.0, preopened_id.1, path)
            .unwrap();
        Ok(())
    }

    fn visit_file(
        &mut self,
        path: &str,
        fd: u32,
        preopened_id: (S::NodeId, S::LinkId),
    ) -> Result<(), u16> {
        let stat = unsafe { wasi::fd_filestat_get(fd) }
            .map_err(|e| e.raw())
            .unwrap();
        if stat.size >= u32::MAX as u64 {
            // ignore too big files
            if self.verbose {
                trace::print(format!("too large file: {} (size {})\n", path, stat.size));
            }
            return Ok(());
        }
        let mut buf = vec![0; stat.size as usize];
        let mut offset = 0;
        loop {
            let read = unsafe {
                wasi::fd_read(
                    fd,
                    &[wasi::Iovec {
                        buf: buf[offset..].as_mut_ptr(),
                        buf_len: buf.len(),
                    }],
                )
            }
            .map_err(|e| e.raw())
            .unwrap();

            offset += read;
            if offset == stat.size as usize {
                break;
            }
        }
        if self.verbose {
            trace::print(format!(
                "pack file: {} under node-id={} (size {})\n",
                path,
                preopened_id.0.ino(),
                buf.len()
            ));
        }
        self.fs
            .create_file(preopened_id.0, preopened_id.1, path, buf)
            .unwrap();
        Ok(())
    }
}

#[derive(Copy, Clone, Hash, Debug)]
pub(crate) struct Error(u16);

impl From<wasi::Errno> for Error {
    #[inline]
    fn from(from: wasi::Errno) -> Self {
        Self(from.raw())
    }
}

impl Error {
    #[inline]
    pub(crate) fn raw(self) -> u16 {
        self.0
    }
}

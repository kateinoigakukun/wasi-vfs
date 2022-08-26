use core::slice;
use std::{
    ffi::{CStr, OsStr},
    mem::MaybeUninit,
    path::Path,
};
use wasi::{
    CiovecArray, Dircookie, Event, Fd, Fdflags, Fdstat, Filedelta, Filesize, Filestat, Fstflags,
    IovecArray, Lookupflags, Oflags, Prestat, PrestatDir, PrestatU, Rights, Size, Subscription,
    Timestamp, RIGHTS_FD_ADVISE, RIGHTS_FD_FILESTAT_GET, RIGHTS_FD_READ, RIGHTS_FD_READDIR,
    RIGHTS_PATH_OPEN,
};

use crate::{
    embed::{Node, NodeDirBody, NodeFileBody, NodeIdTrait, Storage},
    BackingFd, Error, FileSystem, UserFd,
};

pub(crate) unsafe fn fd_advise<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    offset: Filesize,
    len: Filesize,
    advice: i32,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_advise(
                fd as i32,
                offset as i64,
                len as i64,
                advice as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_allocate<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    offset: Filesize,
    len: Filesize,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret =
                wasi::wasi_snapshot_preview1::fd_allocate(fd as i32, offset as i64, len as i64);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_close<S: Storage>(fs: &mut FileSystem<S>, fd: UserFd) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => Ok(fs.embedded_fs.close_file(vfd)?),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_close(fd as i32);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_datasync<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_datasync(fd as i32);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_fdstat_get<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
) -> Result<Fdstat, Error> {
    let fd = fs.get_backing_fd(fd)?;
    let ro_rights = RIGHTS_FD_READ
        | RIGHTS_FD_ADVISE
        | RIGHTS_PATH_OPEN
        | RIGHTS_FD_READDIR
        | RIGHTS_FD_FILESTAT_GET;
    match fd {
        BackingFd::Virtual(vfd) => {
            let stat = fs.embedded_fs.get_fd_stat(vfd)?;
            Ok(stat)
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Fdstat>::uninit();
            let ret =
                wasi::wasi_snapshot_preview1::fd_fdstat_get(fd as i32, rp0.as_mut_ptr() as i32);
            match ret {
                0 => Ok(rp0.assume_init()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_fdstat_set_flags<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    flags: Fdflags,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let entry = fs.embedded_fs.get_fd_entry_mut(vfd)?;
            entry.flags = flags;
            Ok(())
        }
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_fdstat_set_flags(fd as i32, flags as i32);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_fdstat_set_rights<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    fs_rights_base: Rights,
    fs_rights_inheriting: Rights,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_fdstat_set_rights(
                fd as i32,
                fs_rights_base as i64,
                fs_rights_inheriting as i64,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_filestat_get<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
) -> Result<Filestat, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let fd_entry = fs.embedded_fs.get_fd_entry(vfd)?;
            Ok(fs.embedded_fs.get_filestat_from_node_id(fd_entry.node_id))
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Filestat>::uninit();
            let ret =
                wasi::wasi_snapshot_preview1::fd_filestat_get(fd as i32, rp0.as_mut_ptr() as i32);
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Filestat)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_filestat_set_size<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    size: Filesize,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_filestat_set_size(fd as i32, size as i64);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_filestat_set_times<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    atim: Timestamp,
    mtim: Timestamp,
    fst_flags: Fstflags,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_filestat_set_times(
                fd as i32,
                atim as i64,
                mtim as i64,
                fst_flags as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_pread<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    iovs: IovecArray<'_>,
    offset: Filesize,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_pread(
                fd as i32,
                iovs.as_ptr() as i32,
                iovs.len() as i32,
                offset as i64,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Size)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_prestat_get<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
) -> Result<Prestat, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            if let Some(dir) = fs.embedded_fs.get_preopened_dir_path(vfd) {
                let stat = Prestat {
                    tag: wasi::PREOPENTYPE_DIR.raw(),
                    u: PrestatU {
                        dir: PrestatDir {
                            pr_name_len: dir.as_bytes().len(),
                        },
                    },
                };
                Ok(stat)
            } else {
                Err(wasi::ERRNO_BADF.into())
            }
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Prestat>::uninit();
            let ret =
                wasi::wasi_snapshot_preview1::fd_prestat_get(fd as i32, rp0.as_mut_ptr() as i32);
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Prestat)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_prestat_dir_name<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    path: *mut u8,
    path_len: u32,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            if let Some(dir) = fs.embedded_fs.get_preopened_dir_path(vfd) {
                let path = slice::from_raw_parts_mut(path, path_len as usize);
                for (offset, byte) in dir.as_bytes().iter().enumerate() {
                    path[offset] = *byte;
                }
                Ok(())
            } else {
                Err(wasi::ERRNO_BADF.into())
            }
        }
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_prestat_dir_name(
                fd as i32,
                path as i32,
                path_len as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_pwrite<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    iovs: CiovecArray<'_>,
    offset: Filesize,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_pwrite(
                fd as i32,
                iovs.as_ptr() as i32,
                iovs.len() as i32,
                offset as i64,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Size)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_read<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    iovs: IovecArray<'_>,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let node = fs.embedded_fs.get_node(vfd)?;
            match node {
                Node::Dir { .. } => Err(wasi::ERRNO_ISDIR.into()),
                Node::File(body) => {
                    let open = fs.embedded_fs.get_fd_entry(vfd)?;
                    let mut cursor = std::io::Cursor::new(body.content());
                    cursor.set_position(open.offset as u64);
                    let read_bytes = read_bytes(cursor, iovs)?;
                    let open = fs.embedded_fs.get_fd_entry_mut(vfd)?;
                    open.offset += read_bytes;
                    Ok(read_bytes)
                }
            }
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_read(
                fd as i32,
                iovs.as_ptr() as i32,
                iovs.len() as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Size)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_readdir<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    buf: *mut u8,
    buf_len: u32,
    cookie: Dircookie,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let node = fs.embedded_fs.get_node(vfd)?;
            let entries = match node {
                Node::Dir(body) => body.entries(),
                Node::File { .. } => {
                    return Err(wasi::ERRNO_NOTDIR.into());
                }
            };
            let mut bufused = 0;
            let mut current_cookie = cookie;
            let mut buf = buf;
            let buf_len = buf_len as usize;
            for entry in entries.skip(cookie as usize) {
                current_cookie += 1;
                let name_len = entry.name.len();
                let node_id = fs.embedded_fs.get_node_id_by_link(entry.link_id);
                let node_stat = fs.embedded_fs.get_filestat_from_node_id(node_id);
                let dirent = wasi::Dirent {
                    d_next: current_cookie,
                    d_ino: node_id.ino() as u64,
                    d_namlen: name_len as u32,
                    d_type: node_stat.filetype,
                };

                // 1. Copy dirent to the buffer
                let dirent_len = std::mem::size_of::<wasi::Dirent>();
                let dirent_copy_len = std::cmp::min(dirent_len, buf_len - bufused);
                // copy dirent even though the buffer doesn't have enough remaining space
                std::ptr::copy(&dirent as *const _ as *const u8, buf, dirent_copy_len);
                // bail out if the remaining buffer space is not enough
                if dirent_copy_len < dirent_len {
                    // return the number of bytes stored in the buffer
                    return Ok(buf_len);
                }
                buf = buf.add(dirent_copy_len);
                bufused += dirent_copy_len;

                // 2. Copy name string to the buffer
                let name_copy_len = std::cmp::min(name_len, buf_len - bufused);
                // same truncation rule applied as above
                std::ptr::copy(entry.name.as_ptr(), buf, name_copy_len);

                if name_copy_len < name_len {
                    return Ok(buf_len);
                }
                buf = buf.add(name_len);
                bufused += name_copy_len;
            }
            Ok(bufused)
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_readdir(
                fd as i32,
                buf as i32,
                buf_len as i32,
                cookie as i64,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Size)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_renumber<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    to: UserFd,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    let to = fs.get_backing_fd(to)?;
    match (fd, to) {
        (BackingFd::Wasi(fd), BackingFd::Wasi(to)) => {
            let ret = wasi::wasi_snapshot_preview1::fd_renumber(fd as i32, to as i32);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
        (_, _) => todo!(),
    }
}

pub(crate) unsafe fn fd_seek<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    offset: Filedelta,
    whence: i32,
) -> Result<Filesize, Error> {
    let fd = fs.get_backing_fd(fd)?;
    fn compute_new_offset(base: usize, offset: Filedelta) -> Result<usize, Error> {
        let new_offset = if offset >= 0 {
            base + (offset as usize)
        } else {
            let neg_offset = (-offset) as usize;
            if neg_offset <= base {
                base - neg_offset
            } else {
                return Err(wasi::ERRNO_INVAL.into());
            }
        };
        Ok(new_offset)
    }
    match fd {
        BackingFd::Virtual(vfd) => {
            let whence: wasi::Whence = std::mem::transmute(whence as u8);
            match whence {
                wasi::WHENCE_SET => {
                    let fd_entry = fs.embedded_fs.get_fd_entry_mut(vfd)?;
                    fd_entry.offset = offset as usize;
                    Ok(offset as Filesize)
                }
                wasi::WHENCE_CUR => {
                    let fd_entry = fs.embedded_fs.get_fd_entry_mut(vfd)?;
                    let absolute_offset = compute_new_offset(fd_entry.offset, offset)?;
                    fd_entry.offset = absolute_offset;
                    Ok(absolute_offset as Filesize)
                }
                wasi::WHENCE_END => {
                    let fd_entry = fs.embedded_fs.get_fd_entry(vfd)?;
                    let node = fs.embedded_fs.get_node(vfd)?;
                    match node {
                        Node::File(body) => {
                            let content_len = body.content().len();
                            let fd_entry = fs.embedded_fs.get_fd_entry_mut(vfd)?;
                            let absolute_offset = compute_new_offset(content_len, offset)?;
                            fd_entry.offset = absolute_offset;
                            Ok(absolute_offset as Filesize)
                        }
                        Node::Dir { .. } => Err(wasi::ERRNO_INVAL.into()),
                    }
                }
                _ => Err(wasi::ERRNO_INVAL.into()),
            }
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Filesize>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_seek(
                fd as i32,
                offset,
                whence as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Filesize)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_sync<S: Storage>(fs: &mut FileSystem<S>, fd: UserFd) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::fd_sync(fd as i32);
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_tell<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
) -> Result<Filesize, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let node = fs.embedded_fs.get_node(vfd)?;
            let open = fs.embedded_fs.get_fd_entry_mut(vfd)?;
            Ok(open.offset as u64)
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Filesize>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_tell(fd as i32, rp0.as_mut_ptr() as i32);
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Filesize)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn fd_write<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    iovs: CiovecArray<'_>,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::fd_write(
                fd as i32,
                iovs.as_ptr() as i32,
                iovs.len() as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(rp0.assume_init()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_create_directory<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    path: &CStr,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::path_create_directory(
                fd as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_filestat_get<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    flags: Lookupflags,
    path: &CStr,
) -> Result<Filestat, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let path = cstr_to_path(path)?;
            Ok(fs.embedded_fs.get_filestat_at_path(vfd, path)?)
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Filestat>::uninit();
            let ret = wasi::wasi_snapshot_preview1::path_filestat_get(
                fd as i32,
                flags as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Filestat)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_filestat_set_times<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    flags: Lookupflags,
    path: &CStr,
    atim: Timestamp,
    mtim: Timestamp,
    fst_flags: Fstflags,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::path_filestat_set_times(
                fd as i32,
                flags as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
                atim as i64,
                mtim as i64,
                fst_flags as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_link<S: Storage>(
    fs: &mut FileSystem<S>,
    old_fd: UserFd,
    old_flags: Lookupflags,
    old_path: &CStr,
    new_fd: UserFd,
    new_path: &CStr,
) -> Result<(), Error> {
    let old_fd = fs.get_backing_fd(old_fd)?;
    let new_fd = fs.get_backing_fd(new_fd)?;
    match (old_fd, new_fd) {
        (BackingFd::Wasi(old_fd), BackingFd::Wasi(new_fd)) => {
            let ret = wasi::wasi_snapshot_preview1::path_link(
                old_fd as i32,
                old_flags as i32,
                old_path.as_ptr() as i32,
                old_path.to_bytes().len() as i32,
                new_fd as i32,
                new_path.as_ptr() as i32,
                new_path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
        (_, _) => todo!(),
    }
}

pub(crate) unsafe fn path_open<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    dirflags: Lookupflags,
    path: &CStr,
    oflags: Oflags,
    fs_rights_base: Rights,
    fs_rights_inheriting: Rights,
    fdflags: Fdflags,
) -> Result<UserFd, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => {
            let path = cstr_to_path(path)?;
            let new_vfd = fs.embedded_fs.open_file(vfd, path, fdflags)?;
            Ok(fs.issue_user_fd(BackingFd::Virtual(new_vfd)))
        }
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Fd>::uninit();
            let ret = wasi::wasi_snapshot_preview1::path_open(
                fd as i32,
                dirflags as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
                oflags as i32,
                fs_rights_base as i64,
                fs_rights_inheriting as i64,
                fdflags as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => {
                    let new_fd = BackingFd::Wasi(rp0.assume_init());
                    Ok(fs.issue_user_fd(new_fd))
                }
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_readlink<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    path: &CStr,
    buf: *mut u8,
    buf_len: u32,
) -> Result<Size, Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => Err(wasi::ERRNO_INVAL.into()),
        BackingFd::Wasi(fd) => {
            let mut rp0 = MaybeUninit::<Size>::uninit();
            let ret = wasi::wasi_snapshot_preview1::path_readlink(
                fd as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
                buf as i32,
                buf_len as i32,
                rp0.as_mut_ptr() as i32,
            );
            match ret {
                0 => Ok(core::ptr::read(rp0.as_mut_ptr() as i32 as *const Size)),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_remove_directory<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    path: &CStr,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::path_remove_directory(
                fd as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_rename<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    old_path: &CStr,
    new_fd: UserFd,
    new_path: &CStr,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    let new_fd = fs.get_backing_fd(new_fd)?;
    match (fd, new_fd) {
        (BackingFd::Wasi(fd), BackingFd::Wasi(new_fd)) => {
            let ret = wasi::wasi_snapshot_preview1::path_rename(
                fd as i32,
                old_path.as_ptr() as i32,
                old_path.to_bytes().len() as i32,
                new_fd as i32,
                new_path.as_ptr() as i32,
                new_path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
        (_, _) => todo!(),
    }
}

pub(crate) unsafe fn path_symlink<S: Storage>(
    fs: &mut FileSystem<S>,
    old_path: &CStr,
    fd: UserFd,
    new_path: &CStr,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::path_symlink(
                old_path.as_ptr() as i32,
                old_path.to_bytes().len() as i32,
                fd as i32,
                new_path.as_ptr() as i32,
                new_path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn path_unlink_file<S: Storage>(
    fs: &mut FileSystem<S>,
    fd: UserFd,
    path: &CStr,
) -> Result<(), Error> {
    let fd = fs.get_backing_fd(fd)?;
    match fd {
        BackingFd::Virtual(vfd) => todo!(),
        BackingFd::Wasi(fd) => {
            let ret = wasi::wasi_snapshot_preview1::path_unlink_file(
                fd as i32,
                path.as_ptr() as i32,
                path.to_bytes().len() as i32,
            );
            match ret {
                0 => Ok(()),
                _ => Err(Error(ret as u16)),
            }
        }
    }
}

pub(crate) unsafe fn poll_oneoff<S: Storage>(
    fs: &mut FileSystem<S>,
    in_: *const Subscription,
    out: *mut Event,
    nsubscriptions: u32,
) -> Result<Size, Error> {
    Err(wasi::ERRNO_NOTSUP.into())
}

fn read_bytes<R: std::io::Read>(mut src: R, iovs: wasi::IovecArray) -> Result<usize, wasi::Errno> {
    let mut bytes_read = 0;
    for iov in iovs {
        unsafe {
            let buf = slice::from_raw_parts_mut(iov.buf, iov.buf_len);
            bytes_read += src.read(buf).map_err(|_| wasi::ERRNO_IO)?;
        }
    }
    Ok(bytes_read)
}

fn cstr_to_path(path: &CStr) -> Result<&Path, wasi::Errno> {
    let os_str: &OsStr = unsafe { std::mem::transmute(path.to_bytes()) };
    Ok(Path::new(os_str))
}

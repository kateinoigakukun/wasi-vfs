use std::error::Error;

use object_link::AbiVariant;
use structopt::StructOpt;

mod object_link;
mod wrapper;

pub(crate) const WASI_HOOK_FUNCTIONS: &[&str] = &[
    "fd_advise",
    "fd_allocate",
    "fd_close",
    "fd_datasync",
    "fd_fdstat_get",
    "fd_fdstat_set_flags",
    "fd_fdstat_set_rights",
    "fd_filestat_get",
    "fd_filestat_set_size",
    "fd_filestat_set_times",
    "fd_pread",
    "fd_prestat_dir_name",
    "fd_prestat_get",
    "fd_pwrite",
    "fd_read",
    "fd_readdir",
    "fd_renumber",
    "fd_seek",
    "fd_sync",
    "fd_tell",
    "fd_write",
    "path_create_directory",
    "path_filestat_get",
    "path_filestat_set_times",
    "path_link",
    "path_open",
    "path_readlink",
    "path_remove_directory",
    "path_rename",
    "path_symlink",
    "path_unlink_file",
    "poll_oneoff",
];

#[derive(StructOpt)]
pub enum App {
    Wrapper,
    ObjectLink { abi_variant: AbiVariant },
}

impl App {
    pub fn execute(self) -> Result<(), Box<dyn Error>> {
        let witx_paths = witx::phases::snapshot().unwrap();
        match self {
            App::Wrapper => {
                print!("{}", wrapper::generate(&witx_paths));
            }
            App::ObjectLink { abi_variant } => {
                print!("{}", object_link::generate(&witx_paths, abi_variant));
            }
        }
        Ok(())
    }
}

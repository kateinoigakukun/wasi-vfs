use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;
mod module_link;

fn parse_map_dirs(s: &str) -> anyhow::Result<(PathBuf, PathBuf)> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("must contain exactly one double colon ('::')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

#[derive(Debug, StructOpt)]
pub enum App {
    #[structopt(setting(structopt::clap::AppSettings::Hidden))]
    LinkModule {
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        #[structopt(short, parse(from_os_str))]
        output: PathBuf,
    },

    /// Package directories into Wasm module
    Pack {
        /// The input Wasm module's file path.
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Package a host directory into Wasm module at a guest directory
        #[structopt(long = "mapdir", value_name = "GUEST_DIR::HOST_DIR", parse(try_from_str = parse_map_dirs))]
        map_dirs: Vec<(PathBuf, PathBuf)>,

        /// Package a host directory into Wasm module at a guest directory
        #[structopt(long = "dir", value_name = "HOST_DIR::GUEST_DIR", parse(try_from_str = parse_map_dirs))]
        dirs: Vec<(PathBuf, PathBuf)>,

        /// The file path to write the output Wasm module to.
        #[structopt(long, short, parse(from_os_str))]
        output: PathBuf,
    },
}

impl App {
    pub fn execute(self) -> Result<()> {
        match self {
            App::LinkModule { input, .. } => {
                let bytes = std::fs::read(&input)?;
                module_link::link(&bytes);
            }
            App::Pack {
                input,
                map_dirs,
                dirs,
                output,
            } => {
                let wasm_bytes = std::fs::read(&input)?;
                if !map_dirs.is_empty() {
                    eprintln!("warning: --mapdir GUIEST_DIR::HOST_DIR is deprecated, use --dir HOST_DIR::GUEST_DIR instead");
                }

                let mut map_dirs = map_dirs;
                map_dirs.extend(dirs.into_iter().map(|(a, b)| (b, a)));

                let output_bytes = pack(&wasm_bytes, map_dirs)?;
                std::fs::write(output, output_bytes)?;
            }
        }
        Ok(())
    }
}

pub fn pack(wasm_bytes: &[u8], map_dirs: Vec<(PathBuf, PathBuf)>) -> Result<Vec<u8>> {
    std::env::set_var("__WASI_VFS_PACKING", "1");
    let mut wizer = wizer::Wizer::new();
    wizer.allow_wasi(true)?;
    wizer.init_func("wasi_vfs_pack_fs");
    wizer.inherit_stdio(true);
    wizer.inherit_env(true);
    wizer.keep_init_func(true);
    wizer.wasm_bulk_memory(true);
    for (guest_dir, host_dir) in map_dirs {
        wizer.map_dir(guest_dir, host_dir);
    }
    if is_wasi_reactor(&wasm_bytes) {
        wizer.func_rename("_initialize", "__wasi_vfs_rt_init");
    }
    let output_bytes = wizer.run(&wasm_bytes)?;
    Ok(output_bytes)
}

fn is_wasi_reactor(bytes: &[u8]) -> bool {
    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(bytes) {
        let payload = match payload {
            Ok(payload) => payload,
            Err(_) => continue,
        };
        match payload {
            wasmparser::Payload::ExportSection(export) => {
                for entry in export {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => continue,
                    };
                    if entry.name == "_initialize" {
                        return true;
                    }
                }
                return false;
            }
            wasmparser::Payload::End(_) => return false,
            _ => continue,
        }
    }
    false
}

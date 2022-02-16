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
                output,
            } => {
                std::env::set_var("__WASI_VFS_PACKING", "1");
                let mut wizer = wizer::Wizer::new();
                wizer.allow_wasi(true);
                wizer.init_func("wasi_vfs_pack_fs");
                wizer.inherit_stdio(true);
                wizer.inherit_env(true);
                wizer.keep_init_func(true);
                for (guest_dir, host_dir) in map_dirs {
                    wizer.map_dir(guest_dir, host_dir);
                }
                let wasm_bytes = std::fs::read(&input)?;
                let output_bytes = wizer.run(&wasm_bytes)?;
                std::fs::write(output, output_bytes)?;
            }
        }
        Ok(())
    }
}

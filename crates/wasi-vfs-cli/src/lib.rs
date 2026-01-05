use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;
mod module_link;

fn parse_map_dirs(s: &str) -> anyhow::Result<(String, PathBuf)> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("must contain exactly one double colon ('::')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

fn parse_dirs(s: &str) -> anyhow::Result<(PathBuf, String)> {
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
        map_dirs: Vec<(String, PathBuf)>,

        /// Package a host directory into Wasm module at a guest directory
        #[structopt(long = "dir", value_name = "HOST_DIR::GUEST_DIR", parse(try_from_str = parse_dirs))]
        dirs: Vec<(PathBuf, String)>,

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
                    eprintln!(
                        "warning: --mapdir GUIEST_DIR::HOST_DIR is deprecated, use --dir HOST_DIR::GUEST_DIR instead"
                    );
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

pub fn pack(wasm_bytes: &[u8], map_dirs: Vec<(String, PathBuf)>) -> Result<Vec<u8>> {
    unsafe {
        std::env::set_var("__WASI_VFS_PACKING", "1");
    }

    // Use tokio runtime for async wizer
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async { pack_async(wasm_bytes, map_dirs).await })
}

async fn pack_async(wasm_bytes: &[u8], map_dirs: Vec<(String, PathBuf)>) -> Result<Vec<u8>> {
    // Configure WASI
    let mut wasi = wasmtime_wasi::WasiCtxBuilder::new();
    wasi.inherit_stdio();
    wasi.env("__WASI_VFS_PACKING", "1");

    let verbose_env_key = "WASI_VFS_VERBOSE";
    if let Ok(verbose) = std::env::var(verbose_env_key) {
        wasi.env(verbose_env_key, &verbose);
    }

    for (guest_dir, host_dir) in map_dirs {
        wasi.preopened_dir(
            host_dir,
            guest_dir,
            wasmtime_wasi::DirPerms::all(),
            wasmtime_wasi::FilePerms::all(),
        )?;
    }

    // Configure Wasmtime
    let mut config = wasmtime::Config::new();
    config.wasm_bulk_memory(true);
    config.async_support(true);

    let engine = wasmtime::Engine::new(&config)?;
    let mut store = wasmtime::Store::new(&engine, wasi.build_p1());

    // Configure Wizer
    let mut wizer = wasmtime_wizer::Wizer::new();
    wizer.init_func("wasi_vfs_pack_fs");
    wizer.keep_init_func(true);

    // For reactor modules, wasi-vfs needs some special initialization process.
    // 1st pack: Wizer removes `_initialize` and renames `__wasi_vfs_rt_init` to `_initialize`.
    //           And adds `__wasi_vfs_rt_init` as a new export duplicated from `_initialize`.
    // 2nd pack: Wizer removes `_initialize` (which was `__wasi_vfs_rt_init` in the 1st pack)
    //           and renames `__wasi_vfs_rt_init` to `_initialize`.
    //           And adds `__wasi_vfs_rt_init` as a new export duplicated from `_initialize`.
    // 3~n pack: Repeat the 2nd pack.
    if is_wasi_reactor(wasm_bytes) {
        wizer.func_rename("_initialize", "__wasi_vfs_rt_init");
    }

    let output_bytes = wizer
        .run(&mut store, wasm_bytes, async |store, module| {
            // Set up linker with WASI inside the closure
            let mut linker = wasmtime::Linker::new(module.engine());
            wasmtime_wasi::p1::add_to_linker_async(&mut linker, |x| x)?;
            linker.define_unknown_imports_as_traps(module)?;
            linker.instantiate_async(store, module).await
        })
        .await?;

    let output_bytes = copy_export_entry(&output_bytes, "_initialize", "__wasi_vfs_rt_init")?;
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
    return false;
}

/// Copy an export entry to another name.
fn copy_export_entry(bytes: &[u8], source: &str, dest: &str) -> Result<Vec<u8>> {
    let mut module = wasm_encoder::Module::new();

    let parser = wasmparser::Parser::new(0);

    for payload in parser.parse_all(bytes) {
        let payload = payload?;
        match payload {
            wasmparser::Payload::Version { .. } => continue,
            wasmparser::Payload::ExportSection(export) => {
                let mut section = wasm_encoder::ExportSection::new();
                for entry in export {
                    let entry = entry?;
                    section.export(entry.name, translate::export_kind(entry.kind), entry.index);
                    if entry.name == source {
                        section.export(dest, translate::export_kind(entry.kind), entry.index);
                    }
                }
                module.section(&section);
            }
            wasmparser::Payload::End(_) => continue,
            _ => {
                if let Some((id, range)) = payload.as_section() {
                    let raw = wasm_encoder::RawSection {
                        id,
                        data: &bytes[range.start..range.end],
                    };
                    module.section(&raw);
                }
            }
        }
    }

    Ok(module.finish())
}

mod translate {
    pub(crate) fn export_kind(x: wasmparser::ExternalKind) -> wasm_encoder::ExportKind {
        match x {
            wasmparser::ExternalKind::Func => wasm_encoder::ExportKind::Func,
            wasmparser::ExternalKind::Table => wasm_encoder::ExportKind::Table,
            wasmparser::ExternalKind::Memory => wasm_encoder::ExportKind::Memory,
            wasmparser::ExternalKind::Global => wasm_encoder::ExportKind::Global,
            wasmparser::ExternalKind::Tag => wasm_encoder::ExportKind::Tag,
        }
    }
}

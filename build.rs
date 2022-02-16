use std::env;

fn main() {
    let triple = env::var("TARGET").expect("TARGET was not set");
    if !triple.starts_with("wasm32-") {
        println!("wasi-vfs only supports wasm32-unknown-unknown");
        return;
    }
    let wasi_sdk = env::var("WASI_SDK_PATH").expect("WASI_SDK_PATH is not set");
    let mut build = cc::Build::new();
    build
        .compiler(format!("{}/bin/clang", wasi_sdk))
        .archiver(format!("{}/bin/llvm-ar", wasi_sdk))
        .file("src/init.c");

    let trampoline_file = if env::var("CARGO_FEATURE_LEGACY_WASI_LIBC").is_ok() {
        "src/trampoline_generated_legacy_wasi_libc.c"
    } else {
        "src/trampoline_generated.c"
    };
    build.file(trampoline_file);

    println!("cargo:rerun-if-changed=src/init.c");
    println!("cargo:rerun-if-changed={}", trampoline_file);

    build.file("src/embed/linked_storage.c");
    println!("cargo:rerun-if-changed=src/embed/linked_storage.c");

    build.compile("wasi_vfs_c");
}

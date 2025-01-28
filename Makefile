TRAMPOLINE_GEN = WASI_REPO=./crates/wasi-libc-trampoline-bindgen/WASI cargo run --package wasi-libc-trampoline-bindgen --
LIB_WASI_VFS_A = target/wasm32-unknown-unknown/debug/libwasi_vfs.a
.DEFAULT_GOAL = build

.PHONY: generate-trampoline build check

generate-trampoline:
	$(TRAMPOLINE_GEN) wrapper > ./src/trampoline_generated.rs
	$(TRAMPOLINE_GEN) object-link latest > ./src/trampoline_generated.c
	$(TRAMPOLINE_GEN) object-link legacy > ./src/trampoline_generated_legacy_wasi_libc.c

build:
	+WASI_VFS_C_MODE=1 cargo rustc --target wasm32-unknown-unknown --crate-type=staticlib

check: build
	env LIB_WASI_VFS_A=$(LIB_WASI_VFS_A) ./tools/run-make-test.sh

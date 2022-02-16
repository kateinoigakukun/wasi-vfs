TRAMPOLINE_GEN = WASI_REPO=./crates/wasi-libc-trampoline-bindgen/WASI cargo run --package wasi-libc-trampoline-bindgen --

generate-trampoline:
	$(TRAMPOLINE_GEN) wrapper > ./src/trampoline_generated.rs
	$(TRAMPOLINE_GEN) object-link latest > ./src/trampoline_generated.c
	$(TRAMPOLINE_GEN) object-link legacy > ./src/trampoline_generated_legacy_wasi_libc.c

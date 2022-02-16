CC = $(WASI_SDK_PATH)/bin/clang
WASI_VFS_CLI = cargo run -p wasi-vfs-cli --
WASI_RUN = wasmtime
NODE = node

TARGET = wasm32-unknown-wasi

OPTFLAGS ?=
CCFLAGS = -target $(TARGET) $(OPTFLAGS)
LDFLAGS = -target $(TARGET)

RUNMAKE_DIR:=$(dir $(abspath $(lastword $(MAKEFILE_LIST))))

LIB_WASI_VFS ?= $(RUNMAKE_DIR)/../../target/wasm32-unknown-unknown/debug/libwasi_vfs.a

TMPDIR = $(shell mkdir -p .tmp && echo .tmp)

$(TMPDIR)/%.c.o: %.c $(RUNMAKE_DIR)/check.h
	$(CC) -c $(CCFLAGS) $< -o $@

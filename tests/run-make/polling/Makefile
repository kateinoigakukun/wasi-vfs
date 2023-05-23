-include ../tools.mk

objs = $(TMPDIR)/main.c.o

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --mapdir /dev::./dev -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) --mapdir /dev::./dev $(TMPDIR)/main.wasm
# TODO(katei): Currently, poll_oneoff is not supported for vfs fd
#	$(WASI_RUN) --mapdir /dev::./dev $(TMPDIR)/main.packed.wasm

clean:
	rm -rf $(PROG) $(objs)

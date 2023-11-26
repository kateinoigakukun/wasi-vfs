-include ../tools.mk

objs = $(TMPDIR)/main.c.o

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.vfs.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --mapdir /dev::./dev -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) --mapdir /dev::./dev $(TMPDIR)/main.wasm > $(TMPDIR)/main.out
	$(WASI_RUN) --mapdir /dev::./dev $(TMPDIR)/main.vfs.wasm > $(TMPDIR)/main.vfs.out
	diff $(TMPDIR)/main.out $(TMPDIR)/main.vfs.out

clean:
	rm -rf $(PROG) $(objs)

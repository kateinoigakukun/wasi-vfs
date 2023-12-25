-include ../tools.mk

objs = $(TMPDIR)/main.c.o

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --dir ./mnt0::/mnt0 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase1

	$(WASI_VFS_CLI) pack $(TMPDIR)/main.packed.wasm --dir ./mnt1::/mnt1 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase2

	$(WASI_RUN) --dir ./mnt0::/mnt0 --dir ./mnt1::/mnt1 --dir ./mnt1_1::/mnt1 $(TMPDIR)/main.wasm phase3
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.packed.wasm --dir ./mnt1_1::/mnt1 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase3

clean:
	rm -rf $(PROG) $(objs)

-include ../tools.mk

objs = $(TMPDIR)/main.c.o

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --mapdir /mnt0::./mnt0 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase1

	$(WASI_VFS_CLI) pack $(TMPDIR)/main.packed.wasm --mapdir /mnt1::./mnt1 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase2

	$(WASI_RUN) --mapdir /mnt0::./mnt0 --mapdir /mnt1::./mnt1 --mapdir /mnt1::./mnt1_1 $(TMPDIR)/main.wasm phase3
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.packed.wasm --mapdir /mnt1::./mnt1_1 -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm phase3

clean:
	rm -rf $(PROG) $(objs)

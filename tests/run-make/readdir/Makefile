-include ../tools.mk

objs = $(TMPDIR)/main.c.o

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	mkdir -p $(TMPDIR)/mnt/grow-buffer
#   200 entries x 20 chars file name is enough to overflow the initial buffer
	@for i in $(shell seq 200); do touch "$(TMPDIR)/mnt/grow-buffer/dirent_with_long_name_$$i"; done
	wasmtime --mapdir /mnt::$(TMPDIR)/mnt $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --mapdir /mnt::$(TMPDIR)/mnt -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm

clean:
	rm -rf $(PROG) $(objs)

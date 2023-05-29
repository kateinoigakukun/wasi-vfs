-include ../tools.mk

check: $(objs)
	$(CC) $(LDFLAGS) main.c $(LIB_WASI_VFS) -mexec-model=reactor -o $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --mapdir /::./mnt/pack -o $(TMPDIR)/main.packed.wasm
	$(NODE) --experimental-wasi-unstable-preview1 ./check.js $(TMPDIR)/main.packed.wasm

clean:
	rm -rf $(TMPDIR)/*

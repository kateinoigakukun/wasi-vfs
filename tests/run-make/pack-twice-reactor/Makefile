-include ../tools.mk

check: $(objs)
	$(CC) $(LDFLAGS) main.c $(LIB_WASI_VFS) -mexec-model=reactor -o $(TMPDIR)/main.wasm
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm --dir ./mnt0::/mnt0 -o $(TMPDIR)/main.stage0.wasm
	$(NODE) --experimental-wasi-unstable-preview1 ./check.js $(TMPDIR)/main.stage0.wasm 0

	$(WASI_VFS_CLI) pack $(TMPDIR)/main.stage0.wasm --dir ./mnt1::/mnt1 -o $(TMPDIR)/main.stage1.wasm
	$(NODE) --experimental-wasi-unstable-preview1 ./check.js $(TMPDIR)/main.stage1.wasm 1

	$(NODE) --experimental-wasi-unstable-preview1 ./check.js \
	  --dir=./mnt0::/mnt0 --dir=./mnt1::/mnt1 --dir=./mnt1_1::/mnt1 \
	  $(TMPDIR)/main.wasm 2
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.stage1.wasm --dir ./mnt1_1::/mnt1 -o $(TMPDIR)/main.stage2.wasm
	$(NODE) --experimental-wasi-unstable-preview1 ./check.js $(TMPDIR)/main.stage2.wasm 2

clean:
	rm -rf $(TMPDIR)/*

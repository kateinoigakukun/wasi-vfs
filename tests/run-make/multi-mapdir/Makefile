-include ../tools.mk

objs = $(TMPDIR)/main.c.o

define do_check
	$(WASI_RUN) $(2) $(TMPDIR)/main.wasm -- $(1)
	$(WASI_VFS_CLI) pack $(TMPDIR)/main.wasm $(2) -o $(TMPDIR)/main.packed.wasm
	$(WASI_RUN) $(TMPDIR)/main.packed.wasm -- $(1)

endef

check: $(objs)
	$(CC) $(LDFLAGS) $(objs) $(LIB_WASI_VFS) -o $(TMPDIR)/main.wasm
	$(call do_check,simple,--mapdir /mnt1::./mnt1 --mapdir /mnt0::./mnt0)
	$(call do_check,overlap1,--mapdir /mnt0/mnt1::./mnt1 --mapdir /mnt0::./mnt0)
	$(call do_check,overlap2,--mapdir /mnt0::./mnt0 --mapdir /mnt0/mnt1::./mnt1)

clean:
	rm -rf $(PROG) $(objs)

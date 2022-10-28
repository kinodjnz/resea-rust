NAME = kernel

KERNEL_SRCS = $(wildcard kernel/*.rs)

ARCH = cramp32
# ARCH = riscv32
TARGET = $(ARCH)imc-unknown-none-elf
# TARGET = riscv32imc-unknown-none-elf

LLVM_PATH = ../../rust-lang/rust/build/aarch64-apple-darwin/llvm/bin

AS = $(LLVM_PATH)/llvm-mc
OBJDUMP = $(LLVM_PATH)/llvm-objdump
OBJCOPY = $(LLVM_PATH)/llvm-objcopy
LD = $(LLVM_PATH)/ld.lld

all: target/$(NAME).bin target/$(NAME).dump target/kernel.elf

target/CACHEDIR.TAG target/$(TARGET)/release/lib$(NAME).a: $(KERNEL_SRCS)
	cargo build --release

target/boot.o: libsrc/boot.s target/CACHEDIR.TAG
	$(AS) --arch=$(ARCH) --filetype=obj -o $@ $<

target/%.elf: libsrc/linker.ld target/boot.o target/$(TARGET)/release/lib%.a
	$(LD) -T $+ -o $@ -nostdlib --relax

target/%.bin: target/%.elf
	$(OBJCOPY) -O binary $< $@

target/%.hex: target/%.bin
	od -An -tx4 -v $< > $@

target/%.dump: target/%.elf
	$(OBJDUMP) -dSC --mattr=+zbb $< > $@

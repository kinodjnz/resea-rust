NAME = kernel

ARCH = cramp32
# ARCH = riscv32
TARGET = $(ARCH)imc-unknown-none-elf
# TARGET = riscv32imc-unknown-none-elf

KERNEL_SRCS = $(wildcard kernel/*.rs)
KERNEL_ASM_SRCS = $(wildcard kernel/arch/$(ARCH)/*.S)
KERNEL_LD = kernel/arch/$(ARCH)/kernel.ld

LLVM_PATH = ../../rust-lang/rust/build/aarch64-apple-darwin/llvm/bin

AS = $(LLVM_PATH)/llvm-mc
OBJDUMP = $(LLVM_PATH)/llvm-objdump
OBJCOPY = $(LLVM_PATH)/llvm-objcopy
LD = $(LLVM_PATH)/ld.lld

all: target/$(NAME).bin target/$(NAME).dump target/kernel.elf

target/CACHEDIR.TAG target/$(TARGET)/release/lib$(NAME).a: $(KERNEL_SRCS)
	cargo build --features $(ARCH) --release

target/boot.o: $(KERNEL_ASM_SRCS) target/CACHEDIR.TAG
	$(AS) --arch=$(ARCH) --filetype=obj -o $@ $<

target/%.elf: $(KERNEL_LD) target/boot.o target/$(TARGET)/release/lib%.a
	$(LD) -T $+ -o $@ -nostdlib --relax

target/%.bin: target/%.elf
	$(OBJCOPY) -O binary $< $@

target/%.hex: target/%.bin
	od -An -tx4 -v $< > $@

target/%.dump: target/%.elf
	$(OBJDUMP) -dSC --mattr=+zbb $< > $@

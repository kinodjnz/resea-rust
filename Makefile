NAME = kernel

ARCH = cramp32
# ARCH = riscv32
TARGET = $(ARCH)imc-unknown-none-elf

LLVM_PATH = ../../rust-lang/rust/build/aarch64-apple-darwin/llvm/bin
CARGO = ../../rust-lang/rust/build/aarch64-apple-darwin/stage2-tools-bin/cargo

ARCH_DIR = kernel/src/arch/$(ARCH)

INIT = init

KERNEL_SRCS = $(wildcard */src/*.rs $(ARCH_DIR)/*.rs)
KERNEL_ASM_SRCS = $(wildcard $(ARCH_DIR)/*.S)
KERNEL_LD = $(ARCH_DIR)/kernel.ld

KERNEL_ASM_OBJS = $(patsubst %.S,target/%.o,$(notdir $(KERNEL_ASM_SRCS)))

AS = $(LLVM_PATH)/llvm-mc
OBJDUMP = $(LLVM_PATH)/llvm-objdump
OBJCOPY = $(LLVM_PATH)/llvm-objcopy
LD = $(LLVM_PATH)/ld.lld

ASOPT = --arch=$(ARCH) --mattr=+c,+m,+zba,+zbb,+relax --position-independent

all: target/$(NAME).bin target/$(NAME).dump target/kernel.elf

fmt:
	$(CARGO) +nightly fmt

target/CACHEDIR.TAG target/$(TARGET)/release/lib$(NAME).a target/$(TARGET)/release/lib$(INIT).a: $(KERNEL_SRCS)
	$(CARGO) build --features $(ARCH) --release

target/%.o: $(ARCH_DIR)/%.S target/CACHEDIR.TAG
	$(AS) $(ASOPT) --filetype=obj -o $@ $<

target/%.elf: $(KERNEL_LD) $(KERNEL_ASM_OBJS) target/$(TARGET)/release/lib$(INIT).a target/$(TARGET)/release/lib%.a
	$(LD) -T $+ -o $@ -nostdlib --relax

target/%.bin: target/%.elf
	$(OBJCOPY) -O binary $< $@

target/%.hex: target/%.bin
	od -An -tx4 -v $< > $@

target/%.dump: target/%.elf
	$(OBJDUMP) -dSC --mattr=+zba,+zbb,+zbs,+experimental-zbt --print-imm-hex $< > $@

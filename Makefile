NAME = kernel

# ARCH = cramp32
ARCH = riscv32
TARGET = $(ARCH)imc-unknown-none-elf

COMMON_INSN_OPT=+zbb,+xcramp
#COMMON_INSN_OPT=+zbb

CARGO_BUILD_RUSTFLAGS=-C relocation-model=pic -C target-feature=$(COMMON_INSN_OPT),+relax

LLVM_PATH = ../../rust-lang/rust/build/aarch64-apple-darwin/llvm/bin
# CARGO = ../../rust-lang/rust/build/aarch64-apple-darwin/stage2-tools-bin/cargo
CARGO = env CARGO_BUILD_RUSTFLAGS="$(CARGO_BUILD_RUSTFLAGS)" cargo

# ARCH_DIR = kernel/src/arch/$(ARCH)
ARCH_DIR = kernel/src/arch/cramp32

INIT = init

KERNEL_SRCS = $(wildcard */src/*.rs $(ARCH_DIR)/*.rs)
KERNEL_ASM_SRCS = $(wildcard $(ARCH_DIR)/*.S)
KERNEL_LD = $(ARCH_DIR)/kernel.ld

KERNEL_ASM_OBJS = $(patsubst %.S,target/%.o,$(notdir $(KERNEL_ASM_SRCS)))

AS = $(LLVM_PATH)/llvm-mc
OBJDUMP = $(LLVM_PATH)/llvm-objdump
OBJCOPY = $(LLVM_PATH)/llvm-objcopy
LD = $(LLVM_PATH)/ld.lld

INSN_OPT = +zba,+zbs,+zbb,+xcramp
ASOPT = --arch=$(ARCH) --mattr=+c,+m,$(INSN_OPT),+relax

ADDRESS_COMMENT = ./address_comment.rb

all: target/$(NAME).bin target/$(NAME).dump target/kernel.elf

fmt:
	cargo +nightly fmt

target/CACHEDIR.TAG target/$(TARGET)/release/libmemintrinsics.a target/$(TARGET)/release/libmalloc.a target/$(TARGET)/release/lib$(NAME).a target/$(TARGET)/release/lib$(INIT).a: $(KERNEL_SRCS)
	$(CARGO) build --features cramp32 --release
#	$(CARGO) build --features $(ARCH) --release
#	RUSTFLAGS='--emit=llvm-ir' $(CARGO) build --features $(ARCH) --release

target/%.o: $(ARCH_DIR)/%.S target/CACHEDIR.TAG
	$(AS) $(ASOPT) --filetype=obj -o $@ $<

target/%.elf: $(KERNEL_LD) $(KERNEL_ASM_OBJS) target/$(TARGET)/release/libmemintrinsics.a target/$(TARGET)/release/libmalloc.a target/$(TARGET)/release/lib$(INIT).a target/$(TARGET)/release/lib%.a
	$(LD) -T $+ -o $@ -nostdlib --relax

target/%.bin: target/%.elf
	$(OBJCOPY) -O binary $< $@

target/%.hex: target/%.bin
	od -An -tx4 -v $< > $@

target/%.dump.nocomment: target/%.elf
	$(OBJDUMP) -dSC --mattr=$(INSN_OPT) --print-imm-hex $< > $@

target/%.dump: target/%.dump.nocomment
	$(ADDRESS_COMMENT) < $< > $@

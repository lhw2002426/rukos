rust_lib_name := ruxglibc
rust_lib := target/$(TARGET)/$(MODE)/lib$(rust_lib_name).a

libgcc :=

CFLAGS += -nostdinc -fno-builtin -ffreestanding -Wall
CFLAGS += -isystem$(CURDIR)/$(inc_dir)
LDFLAGS += -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT)

ifeq ($(MODE), release)
  CFLAGS += -O3 
else ifeq ($(MODE), reldebug)
  CFLAGS += -Og -g
else
  CFLAGS += -Og -g
endif

ifeq ($(ARCH), x86_64)
  LDFLAGS += --no-relax
  CFLAGS += -mno-red-zone
else ifeq ($(ARCH), riscv64)
  CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
endif

ifeq ($(findstring fp_simd,$(FEATURES)),)
  ifeq ($(ARCH), x86_64)
    CFLAGS += -mno-sse
  else ifeq ($(ARCH), aarch64)
    CFLAGS += -mgeneral-regs-only
  endif
else
  ifeq ($(ARCH), riscv64)
    # for compiler-rt fallbacks like `__addtf3`, `__multf3`, ...
    libgcc := $(shell $(CC) -print-libgcc-file-name)
  endif
endif

-include $(APP)/axbuild.mk  # override `app-objs`

app-objs := $(addprefix $(APP)/,$(app-objs))

$(app-objs): build_glibc prebuild

$(APP)/%.o: $(APP)/%.c build_glibc
	$(call run_cmd,$(CC),$(CFLAGS) $(APP_CFLAGS) -c -o $@ $<)

$(rust_lib): _cargo_build

$(OUT_ELF): $(rust_lib) $(libgcc)
	@printf "    $(CYAN_C)Linking$(END_C) $(OUT_ELF)\n"
	$(call run_cmd,$(LD),$(LDFLAGS)  $(rust_lib) $(libgcc) -o $@)

$(APP)/axbuild.mk: ;

.PHONY: build_glibc

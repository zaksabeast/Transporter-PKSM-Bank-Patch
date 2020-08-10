CARGO = cargo
RELEASE_BUILD = target/arm-unknown-linux-gnueabi/release/transporter_pksm_patch
OBJCOPY = $(DEVKITARM)/bin/arm-none-eabi-objcopy

.PHONY: all build clean

all: build

build:
	@$(CARGO) +nightly build --release --no-default-features
	@$(OBJCOPY) -I elf32-little --dump-section .text=$(RELEASE_BUILD).text --dump-section .rodata=$(RELEASE_BUILD).rodata $(RELEASE_BUILD)
	@node make-ips.js

clean:
	@$(CARGO) clean
	@rm code.ips
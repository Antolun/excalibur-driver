# SPDX-License-Identifier: GPL-3.0-or-later
# Makefile for Casper Excalibur Linux Kernel Driver and Control Utility

MODULE_NAME   := excalibur
VERSION       := 2.0.0
KDIR          ?= /lib/modules/$(shell uname -r)/build
PWD           ?= $(CURDIR)
KERNEL_DIR    := $(PWD)/kernel
RUST_CORE_DIR := $(PWD)/rust_core
CLI_DIR       := $(PWD)/cli

# Automatic LLVM/Clang detection if kernel was built with Clang
ifneq ($(wildcard $(KDIR)/.config),)
  ifneq ($(shell grep -s "CONFIG_CC_IS_CLANG=y" $(KDIR)/.config),)
    LLVM ?= 1
    CC   ?= clang
  endif
endif

RUSTC ?= rustc
CARGO ?= cargo

RUSTC_KERNEL_FLAGS := --edition 2021 \
                      --crate-type staticlib \
                      --emit=obj \
                      -C panic=abort \
                      -C opt-level=3 \
                      -C code-model=kernel \
                      -C no-redzone=y \
                      -C force-unwind-tables=no

.PHONY: all modules cli clean install uninstall dkms-install dkms-uninstall load unload status

all: modules cli

# Compile Rust Core static object for the kernel module
$(KERNEL_DIR)/excalibur_core_rust.o: $(RUST_CORE_DIR)/src/*.rs
	@echo "  [RUSTC] Compiling Rust core for kernel..."
	$(RUSTC) $(RUSTC_KERNEL_FLAGS) $(RUST_CORE_DIR)/src/lib.rs -o $@
	@echo "cmd_$(KERNEL_DIR)/excalibur_core_rust.o := $(RUSTC)" > $(KERNEL_DIR)/.excalibur_core_rust.o.cmd

# Build the kernel module via Kbuild
modules: $(KERNEL_DIR)/excalibur_core_rust.o
	@echo "  [KBUILD] Building kernel module $(MODULE_NAME).ko..."
	$(MAKE) -C $(KDIR) M=$(KERNEL_DIR) $(if $(LLVM),LLVM=$(LLVM),) modules

# Build Userspace Rust CLI
cli:
	@if [ -d "$(CLI_DIR)" ]; then \
		echo "  [CARGO] Building excalibur-ctl CLI..."; \
		$(CARGO) build --release --manifest-path $(CLI_DIR)/Cargo.toml; \
	fi

clean:
	@echo "  [CLEAN] Cleaning build artifacts..."
	$(MAKE) -C $(KDIR) M=$(KERNEL_DIR) clean || true
	rm -f $(KERNEL_DIR)/excalibur_core_rust.o $(KERNEL_DIR)/.excalibur_core_rust.o.cmd $(KERNEL_DIR)/excalibur.ko
	if [ -d "$(CLI_DIR)" ]; then $(CARGO) clean --manifest-path $(CLI_DIR)/Cargo.toml || true; fi
	if [ -d "$(RUST_CORE_DIR)" ]; then $(CARGO) clean --manifest-path $(RUST_CORE_DIR)/Cargo.toml || true; fi

install: modules
	@echo "  [INSTALL] Installing kernel module..."
	install -d /lib/modules/$(shell uname -r)/extra
	install -m 644 $(KERNEL_DIR)/$(MODULE_NAME).ko /lib/modules/$(shell uname -r)/extra/
	depmod -a $(shell uname -r)
	@if [ -f "$(CLI_DIR)/target/release/excalibur-ctl" ]; then \
		echo "  [INSTALL] Installing excalibur-ctl to /usr/local/bin..."; \
		install -m 755 $(CLI_DIR)/target/release/excalibur-ctl /usr/local/bin/excalibur-ctl; \
	fi
	@if [ -f "$(PWD)/udev/99-excalibur.rules" ]; then \
		echo "  [INSTALL] Installing udev rules..."; \
		install -m 644 $(PWD)/udev/99-excalibur.rules /etc/udev/rules.d/99-excalibur.rules; \
		udevadm control --reload-rules || true; \
		udevadm trigger || true; \
	fi
	@echo "Installation complete."

uninstall:
	@echo "  [UNINSTALL] Removing kernel module and utilities..."
	rm -f /lib/modules/$(shell uname -r)/extra/$(MODULE_NAME).ko
	depmod -a $(shell uname -r)
	rm -f /usr/local/bin/excalibur-ctl
	rm -f /etc/udev/rules.d/99-excalibur.rules
	udevadm control --reload-rules || true
	@echo "Uninstallation complete."

load:
	@echo "  [MODPROBE] Loading $(MODULE_NAME) module..."
	-rmmod $(MODULE_NAME) 2>/dev/null || true
	insmod $(KERNEL_DIR)/$(MODULE_NAME).ko

unload:
	@echo "  [RMMOD] Unloading $(MODULE_NAME) module..."
	rmmod $(MODULE_NAME)

status:
	@echo "=== Module Info ==="
	@lsmod | grep $(MODULE_NAME) || echo "Module not loaded"
	@echo "\n=== Hardware Monitoring ==="
	@for d in /sys/class/hwmon/hwmon*; do \
		if [ "$$(cat $$d/name 2>/dev/null)" = "excalibur_wmi" ]; then \
			echo "HWMON: $$d"; \
			[ -f $$d/fan1_input ] && echo "  CPU Fan: $$(cat $$d/fan1_input) RPM"; \
			[ -f $$d/fan2_input ] && echo "  GPU Fan: $$(cat $$d/fan2_input) RPM"; \
			[ -f $$d/pwm1 ] && echo "  Power Plan: $$(cat $$d/pwm1)"; \
		fi; \
	done
	@echo "\n=== Platform Profile ==="
	@if [ -f /sys/firmware/acpi/platform_profile ]; then \
		echo "Current Profile: $$(cat /sys/firmware/acpi/platform_profile 2>/dev/null)"; \
		echo "Choices: $$(cat /sys/firmware/acpi/platform_profile_choices 2>/dev/null)"; \
	fi
	@echo "\n=== Keyboard LEDs ==="
	@for l in /sys/class/leds/excalibur::kbd_backlight-*; do \
		if [ -d "$$l" ]; then \
			echo "Zone: $$(basename $$l)"; \
			echo "  Brightness: $$(cat $$l/brightness 2>/dev/null)/$$(cat $$l/max_brightness 2>/dev/null)"; \
			echo "  Mode: $$(cat $$l/mode 2>/dev/null)"; \
		fi; \
	done

dkms-install:
	@echo "  [DKMS] Registering and building $(MODULE_NAME)-$(VERSION)..."
	mkdir -p /usr/src/$(MODULE_NAME)-$(VERSION)
	cp -r $(PWD)/* /usr/src/$(MODULE_NAME)-$(VERSION)/
	dkms add -m $(MODULE_NAME) -v $(VERSION)
	dkms build -m $(MODULE_NAME) -v $(VERSION)
	dkms install -m $(MODULE_NAME) -v $(VERSION) --force

dkms-uninstall:
	@echo "  [DKMS] Removing $(MODULE_NAME)-$(VERSION)..."
	dkms remove -m $(MODULE_NAME) -v $(VERSION) --all || true
	rm -rf /usr/src/$(MODULE_NAME)-$(VERSION)

luppo-build:
	@echo "  [LUPPO] Building package with Luppo (lopec.xml + COMAR)..."
	luppo build lopec.xml --no-sandbox --ignore-dependency


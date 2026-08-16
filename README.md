# Casper Excalibur Kernel Driver for LupuS

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Linux Kernel](https://img.shields.io/badge/Kernel-5.10%20..%207.x-green.svg)](https://kernel.org/)
[![DKMS](https://img.shields.io/badge/DKMS-Supported-brightgreen.svg)](https://github.com/dell/dkms)
[![Luppo](https://img.shields.io/badge/Luppo-Package-blueviolet.svg)](https://github.com/solzic0/luppo)

An advanced, out-of-tree Linux ACPI/WMI kernel driver and control utility for **Casper Excalibur** gaming laptops (G770, G870, G900, G911, G650, G750, G670, etc.).

Featuring a **`#![no_std]` Rust core**, standard Linux **`platform_profile`** power plan management, **`hwmon`** fan monitoring/control, per-zone **RGB keyboard & corner LED** lighting, automated **DKMS** rebuild support, and native **Luppo / COMAR** package integration.

---

## 🌟 Key Features

- 🦀 **`no_std` Rust Core (`excalibur_core`)**:
  - Encapsulates low-level WMI protocol packet formatting, 32-bit hardware LED word packing/unpacking, fan speed endianness decoding, and strict safety validation in safe, zero-overhead Rust.
- ⚡ **Standard Linux Power Profiles (`platform_profile`)**:
  - Fully integrated with the kernel's ACPI `platform_profile` subsystem (`/sys/firmware/acpi/platform_profile`).
  - Seamlessly supported by **KDE Power Management**, **GNOME Control Center**, `power-profiles-daemon`, `tlp`, and `tuned`.
  - 4 Hardware Modes:
    1. **Performance / Turbo** (`EXCALIBUR_PLAN_HIGH_POWER`)
    2. **Gaming / Balanced** (`EXCALIBUR_PLAN_GAMING`)
    3. **Quiet / Office / Silent** (`EXCALIBUR_PLAN_TEXT_MODE`)
    4. **Eco / Low Power / Battery Saver** (`EXCALIBUR_PLAN_LOW_POWER`)
- 🌈 **Per-Zone RGB Keyboard & Corner LED Control**:
  - Registered under the standard Linux LED class subsystem (`/sys/class/leds/excalibur::kbd_backlight-*`).
  - 4 Independent Lighting Zones:
    - `left` (Zone 0x05)
    - `middle` (Zone 0x04)
    - `right` (Zone 0x03)
    - `corners` (Zone 0x07)
  - 8 Hardware Animation Modes: `static`, `fade` (breathing), `blink`, `heartbeat`, `wave`, `random`, `rainbow`, `off`.
  - Brightness levels (`0` = Off, `1` = Low, `2` = High).
  - 24-bit hex color support (`#RRGGBB`, `RRGGBB`, `0xRRGGBB`).
- 🌪️ **Hardware Monitoring (`hwmon`)**:
  - Real-time RPM readout for CPU (`fan1_input`) and GPU (`fan2_input`) fans.
  - Automatic quirk matching for older generation laptops (swapped big-endian byte order on 10th-gen Intel) vs. modern native little-endian models.
- 🔄 **DKMS (Dynamic Kernel Module Support)**:
  - Automatically compiles and installs the driver upon kernel upgrades.
- 📦 **Luppo & COMAR Ecosystem Integration**:
  - Native recipe definitions (`lopec.xml`, `lopec.kdl`, `actions.py`).
  - COMAR `System.Package` and `System.Service` integration (`package.py`, `package.kdl`, `service.py`, `service.kdl`).
- 🛠️ **Command-Line Interface (`excalibur-ctl`)**:
  - High-performance, zero-dependency CLI written in Rust for checking status, switching power plans, tuning RGB lighting, and streaming fan speeds in real time.
- 🔒 **Udev Rule (`99-excalibur.rules`)**:
  - Grants non-root users permission to control RGB effects and power plans without requiring `sudo`.

---

## 💻 Hardware Compatibility

| Model Series | DMI Identification | Fan RPM Decode | RGB Lighting |
|:---|:---|:---|:---|
| **Excalibur G770** | `EXCALIBUR G770` | Little-Endian (Native) | 3 Keyboard Zones + Corners |
| **Excalibur G870** | `EXCALIBUR G870` | Little-Endian (Native) | 3 Keyboard Zones + Corners |
| **Excalibur G900** | `EXCALIBUR G900` | Big-Endian (Byte-Swapped) | 3 Keyboard Zones + Corners |
| **Excalibur G911** | `EXCALIBUR G911` | Little-Endian (Native) | 3 Keyboard Zones + Corners |
| **Excalibur G650 / G670 / G750** | `EXCALIBUR G...` | Big-Endian (Byte-Swapped) | 3 Keyboard Zones + Corners |
| *Generic Excalibur WMI* | GUID `644C5791-...` | Auto / Default | 3 Keyboard Zones + Corners |

---

## 📋 System Requirements

- **Linux Kernel**: 5.10+ (Tested and verified on 6.x and 7.x kernels)
- **Rust Toolchain**: `rustc` and `cargo` (1.75+)
- **Build Tools**: `make`, `gcc` or `clang` / `llvm`
- **Kernel Headers**: `linux-headers` package corresponding to your active kernel
- **Optional**: `dkms` (for automatic kernel rebuilds), `luppo` & `comar` (for LupuS packaging)

---

## 🔨 Build & Installation

### Option 1: Direct Build & Installation (Makefile)

```bash
# 1. Clone the repository
git clone https://github.com/solzic0/excalibur-driver.git
cd excalibur-driver

# 2. Build the kernel module and userspace CLI
make all

# 3. Install the module, CLI binary, and udev rules
sudo make install

# 4. Load the kernel module
sudo make load
```

### Option 2: Automated DKMS Installation

Register the driver with DKMS so it automatically rebuilds whenever your system updates its Linux kernel:

```bash
sudo make dkms-install
sudo modprobe excalibur
```

### Option 3: Luppo Package Manager

For distributions using the **Luppo** package manager:

```bash
# Build the package from lopec.xml recipe
luppo build lopec.xml --no-sandbox

# Or using the Makefile target
make luppo-build
```

---

## 🎮 CLI Control Utility (`excalibur-ctl`)

Once installed, the `excalibur-ctl` command is available system-wide.

### 1. View System Status
Displays CPU/GPU fan RPM, active power profile, and all RGB zone states:
```bash
excalibur-ctl status
```

### 2. Manage Power Profiles
```bash
# Get the current power profile
excalibur-ctl profile get

# Set High Power / Performance mode
excalibur-ctl profile set performance

# Set Balanced / Gaming mode
excalibur-ctl profile set gaming

# Set Quiet / Office mode
excalibur-ctl profile set quiet

# Set Battery Saver / Eco mode
excalibur-ctl profile set low_power
```

### 3. Configure Keyboard RGB Lighting
```bash
# Set all zones to static red at maximum brightness
excalibur-ctl rgb --zone all --color FF0000 --mode static --brightness 2

# Enable rainbow wave animation across all zones
excalibur-ctl rgb --mode rainbow

# Set the left zone to breathing/fade cyan
excalibur-ctl rgb --zone left --color 00FFFF --mode fade --brightness 2

# Turn off corner LEDs
excalibur-ctl rgb --zone corners --brightness 0
```

### 4. Monitor Fan Speeds
```bash
# Single snapshot
excalibur-ctl fan

# Live continuous monitor (1-second refresh)
excalibur-ctl fan --watch
```

---

## ⚙️ Sysfs Interface Reference

The driver exposes standard Linux kernel interfaces that can be scripted directly:

### 1. ACPI Platform Profile
```bash
# Read active profile
cat /sys/firmware/acpi/platform_profile

# List available choices
cat /sys/firmware/acpi/platform_profile_choices

# Switch profile (performance, balanced, quiet, low-power)
echo "performance" | sudo tee /sys/firmware/acpi/platform_profile
```

### 2. LED Subsystem (`/sys/class/leds/excalibur::kbd_backlight-*`)
- `brightness` (0, 1, 2)
- `max_brightness` (2)
- `color` (Write-only 6-character hex string, e.g., `FF0055`)
- `mode` (Read/Write mode: `static`, `fade`, `blink`, `heartbeat`, `wave`, `random`, `rainbow`, `off`)
- `available_modes` (Read-only list of valid modes)
- `raw` (Write-only 32-bit hex word for hardware debugging)

*Example:*
```bash
echo "FF0088" > /sys/class/leds/excalibur::kbd_backlight-middle/color
echo "fade"   > /sys/class/leds/excalibur::kbd_backlight-middle/mode
echo "2"      > /sys/class/leds/excalibur::kbd_backlight-middle/brightness
```

### 3. Hardware Monitoring (`/sys/class/hwmon/hwmonX/`)
- `fan1_input`: CPU Fan RPM
- `fan1_label`: `cpu_fan`
- `fan2_input`: GPU Fan RPM
- `fan2_label`: `gpu_fan`
- `pwm1`: Power plan numeric ID (1 = High Power, 2 = Gaming, 3 = Quiet, 4 = Low Power)

---

## 📦 Luppo & COMAR Architecture

This project is fully integrated with the **LupuS / Luppo** package manager and **COMAR** configuration manager:

- **[`lopec.xml`](lopec.xml)**: XML package specification containing source, package metadata, dependencies, and COMAR triggers.
- **[`lopec.kdl`](lopec.kdl)**: Modern KDL package recipe alternative.
- **[`actions.py`](actions.py)**: Python / ActionsAPI build and installation routines.
- **[`comar/package.py`](comar/package.py)** & **[`comar/package.kdl`](comar/package.kdl)**: `System.Package` lifecycle handler (handles DKMS registration, `depmod`, udev reload, and module probing).
- **[`comar/service.py`](comar/service.py)** & **[`comar/service.kdl`](comar/service.kdl)**: `System.Service` D-Bus service provider for starting, stopping, and checking driver status.

---

## 🗑️ Uninstallation

To completely remove the driver, DKMS registration, CLI binary, and udev rules from your system:

```bash
# Remove from DKMS (if installed via DKMS)
sudo make dkms-uninstall

# Remove kernel module, CLI binary, and udev rules
sudo make uninstall
```

---

## 📄 License & Authors

- **Authors**: Solzic0 & Antigravity
- **License**: [GPL-2.0-or-later](LICENSE)

#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# Actions file for Luppo package builder (excalibur-driver)
#

import os
import shutil

from luppo.actionsapi import shelltools
from luppo.actionsapi import cargotools
from luppo.actionsapi import luppotools
from luppo.actionsapi import get

NoStrip = ["/lib/modules"]

def setup():
    pass

def build():
    # 1. Compile Rust core and kernel module
    shelltools.system("make modules")
    # 2. Compile Userspace Rust CLI
    shelltools.system("make cli")

def install():
    install_dir = get.installDIR()

    # 1. Install CLI utility
    luppotools.dobin("cli/target/release/excalibur-ctl")

    # 2. Install DKMS source directory
    dkms_dir = os.path.join(install_dir, "usr", "src", "excalibur-wmi-2.0.0")
    shelltools.makedirs(dkms_dir)
    shelltools.copytree("kernel", os.path.join(dkms_dir, "kernel"))
    shelltools.copytree("rust_core", os.path.join(dkms_dir, "rust_core"))
    shelltools.copy("Makefile", dkms_dir)
    shelltools.copy("dkms.conf", dkms_dir)

    # 3. Install precompiled kernel module for current running kernel
    kver = get.curKERNEL() if hasattr(get, "curKERNEL") and get.curKERNEL() else os.uname().release
    mod_dir = os.path.join(install_dir, "lib", "modules", kver, "extra")
    shelltools.makedirs(mod_dir)
    shelltools.copy("kernel/excalibur.ko", os.path.join(mod_dir, "excalibur.ko"))

    # 4. Install udev rule
    udev_dir = os.path.join(install_dir, "etc", "udev", "rules.d")
    shelltools.makedirs(udev_dir)
    shelltools.copy("udev/99-excalibur.rules", os.path.join(udev_dir, "99-excalibur.rules"))

    # 5. Install documentation
    luppotools.dodoc("README.md", "dkms.conf")

#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# Actions file for Luppo package builder (excalibur-driver)
#

import os
from luppo.actionsapi import shelltools
from luppo.actionsapi import luppotools
from luppo.actionsapi import get

WorkDir = "."
NoStrip = ["/lib/modules"]

def setup():
    pass

def build():
    pass

def install():
    src_dir = os.environ.get("EXCALIBUR_DRIVER_SRC_DIR", os.getcwd())
    if not os.path.isdir(os.path.join(src_dir, "kernel")):
        candidates = [
            src_dir,
            os.getcwd(),
            "/home/solzic0/Projeler/LupuS/excalibur-driver",
        ]
        for c in candidates:
            if os.path.isdir(os.path.join(c, "kernel")):
                src_dir = c
                break

    install_dir = get.installDIR()

    # 1. Install CLI utility
    cli_bin = os.path.join(src_dir, "cli/target/release/excalibur-ctl")
    if os.path.isfile(cli_bin):
        luppotools.dobin(cli_bin)
    else:
        raise RuntimeError(f"excalibur-ctl binary not found at {cli_bin}!")

    # 2. Install DKMS source directory
    dkms_dir = os.path.join(install_dir, "usr", "src", "excalibur-wmi-2.0.0")
    shelltools.makedirs(dkms_dir)
    shelltools.copytree(os.path.join(src_dir, "kernel"), os.path.join(dkms_dir, "kernel"))
    shelltools.copytree(os.path.join(src_dir, "rust_core"), os.path.join(dkms_dir, "rust_core"))
    shelltools.copy(os.path.join(src_dir, "Makefile"), dkms_dir)
    shelltools.copy(os.path.join(src_dir, "dkms.conf"), dkms_dir)

    # 3. Install precompiled kernel module for current running kernel
    kver = get.curKERNEL() if hasattr(get, "curKERNEL") and get.curKERNEL() else os.uname().release
    mod_dir = os.path.join(install_dir, "lib", "modules", kver, "extra")
    shelltools.makedirs(mod_dir)
    ko_file = os.path.join(src_dir, "kernel/excalibur.ko")
    if os.path.isfile(ko_file):
        shelltools.copy(ko_file, os.path.join(mod_dir, "excalibur.ko"))
    else:
        raise RuntimeError(f"excalibur.ko kernel module not found at {ko_file}!")

    # 4. Install udev rule
    udev_file = os.path.join(src_dir, "udev/99-excalibur.rules")
    if os.path.isfile(udev_file):
        udev_dir = os.path.join(install_dir, "etc", "udev", "rules.d")
        shelltools.makedirs(udev_dir)
        shelltools.copy(udev_file, os.path.join(udev_dir, "99-excalibur.rules"))

    # 5. Install documentation
    readme_path = os.path.join(src_dir, "README.md")
    dkms_conf_path = os.path.join(src_dir, "dkms.conf")
    if os.path.isfile(readme_path):
        luppotools.dodoc(readme_path)
    if os.path.isfile(dkms_conf_path):
        luppotools.dodoc(dkms_conf_path)

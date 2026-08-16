#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# COMAR System.Package script for excalibur-driver
#

import os

def postInstall(fromVersion, fromRelease, toVersion, toRelease):
    # 1. Register and build with DKMS for automatic kernel updates
    try:
        os.system("dkms add -m excalibur-wmi -v 2.0.0 2>/dev/null || true")
        os.system("dkms build -m excalibur-wmi -v 2.0.0 2>/dev/null || true")
        os.system("dkms install -m excalibur-wmi -v 2.0.0 --force 2>/dev/null || true")
    except:
        pass

    # 2. Update kernel module dependencies
    try:
        os.system("depmod -a")
    except:
        pass

    # 3. Reload udev rules for permissions
    try:
        os.system("udevadm control --reload-rules || true")
        os.system("udevadm trigger || true")
    except:
        pass

    # 4. Automatically load kernel driver
    try:
        os.system("modprobe excalibur 2>/dev/null || true")
    except:
        pass

def preRemove():
    # Unload driver before removal
    try:
        os.system("rmmod excalibur 2>/dev/null || true")
    except:
        pass

    # Unregister from DKMS
    try:
        os.system("dkms remove -m excalibur-wmi -v 2.0.0 --all 2>/dev/null || true")
    except:
        pass

def postRemove():
    # Update module dependencies and udev
    try:
        os.system("depmod -a")
        os.system("udevadm control --reload-rules || true")
    except:
        pass

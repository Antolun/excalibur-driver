#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# COMAR System.Service script for excalibur-driver
#

from comar.service import *
import os

serviceType = "local"
serviceDefault = "on"
serviceDesc = _({
    "en": "Casper Excalibur Hardware Driver & Control Service",
    "tr": "Casper Excalibur Donanım Sürücüsü ve Kontrol Servisi"
})

@synchronized
def start():
    # 1. Ensure kernel module is loaded
    if not os.path.exists("/sys/bus/wmi/drivers/excalibur-wmi"):
        startService(command="/sbin/modprobe",
                     args="excalibur",
                     donotify=True)
    else:
        notify("System.Service.changed", "started")

@synchronized
def stop():
    stopService(command="/sbin/rmmod",
                args="excalibur",
                donotify=True)

def status():
    # Check if excalibur driver is active
    if os.path.exists("/sys/bus/wmi/drivers/excalibur-wmi"):
        return True
    
    try:
        with open("/proc/modules", "r") as f:
            for line in f:
                if line.startswith("excalibur "):
                    return True
    except:
        pass
    
    return False

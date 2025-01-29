#!/bin/bash
#
# load cyw43 wireless firmware w/ bluetooth firmware 
# update the const values in main.rs if you change these locations.
#
# this version uses picotool, make sure its installed/built and the Pico2W is in BOOTSEL mode
set -e

# 0x100000000 = ROM base
# Load Wifi firmware @ 3MiB mark, 256KiB hole
/opt/picotool load --ignore-partitions -t bin include/cyw43-firmware/43439A0.bin -o 0x10380000

# Load Bluetooth firmware @ 1MiB + 256KiB, 16 KiB hole
/opt/picotool load --ignore-partitions -t bin include/cyw43-firmware/43439A0_btfw.bin -o 0x103C0000

# load CLM firmware @ 1MiB + 256KiB + 16 KiB
/opt/picotool load --ignore-partitions -t bin include/cyw43-firmware/43439A0_clm.bin -o 0x103C4000

echo "[picotool] Firmware flashed successfully!"
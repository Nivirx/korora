#!/bin/bash
#
# load cyw43 wireless firmware w/ bluetooth firmware 
# update the const values in main.rs if you change these locations.
#
set -e

# 0x100000000 = ROM base
# Load Wifi firmware @ 3MiB mark, 256KiB hole
probe-rs download ./include/cyw43-firmware/43439A0.bin \
  --binary-format bin --chip RP235x --verify --base-address 0x10300000

# Load Bluetooth firmware @ 1MiB + 256KiB, 16 KiB hole
probe-rs download ./include/cyw43-firmware/43439A0_btfw.bin \
  --binary-format bin --chip RP235x --verify --base-address 0x10340000

# load CLM firmware @ 1MiB + 256KiB + 16 KiB
probe-rs download ./include/cyw43-firmware/43439A0_clm.bin \
  --binary-format bin --chip RP235x --verify --base-address 0x10344000

echo "Firmware flashed successfully!"

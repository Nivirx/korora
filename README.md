# Korora RTOS

This project is a hobby RTOS (and my Rust scratch pad for the Pico2W).

It includes all of the `knurling-rs` tooling as showcased in <https://github.com/knurling-rs/app-template> (`defmt`, `defmt-rtt`, `panic-probe`, `flip-link`) to make development as easy as possible.

`probe-rs` in SWD mode is configured as the default runner, so you can run your binary with

```sh
cargo run --release
```

## Why Korora

 [Wikipedia](https://en.wikipedia.org/wiki/Little_penguin)

## Requirements
  
- The standard Rust tooling (cargo, rustup) which you can install from <https://rustup.rs/>

- Toolchain support for the cortex-m33+ processors in the rp2350 (thumbv8m.main-none-eabihf)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

- A [`probe-rs` compatible](https://probe.rs/docs/getting-started/probe-setup/) probe

- A [`probe-rs` installation](https://probe.rs/docs/getting-started/installation/) *or* `ryan-summers:feature/rp2350-flashing` branch of `https://github.com/ryan-summers/probe-rs.git` installed for probe-rs (*currently pending a PR into the probe-rs mainline*)
 *or*
- the [`pico-sdk` repo](https://github.com/raspberrypi/pico-sdk) and [`picotool` repo](https://github.com/raspberrypi/picotool) built, we currently expect the binary at `/opt/picotool`

`NOTE: I use a J-link SEGGER probe to connect, when connecting to ARMv8 targets in SWD mode I need to launch Ozone/JlinkExe before using probe-rs to flash or debug the connected RP235x, otherwise I get DAP_FAULT's`

## Installation of development dependencies

```sh
rustup target install thumbv8m.main-none-eabihf
cargo install flip-link


# Installs the probe-rs tools, including probe-rs run, our recommended default runner
cargo install --locked probe-rs-tools
# or to use a custom git repo (i.e for features not in mainline)
cargo install --git https://github.com/ryan-summers/probe-rs.git --branch feature/rp2350-flashing probe-rs-tools --locked 

```

If you get the error ``binary `cargo-embed` already exists`` during installation of probe-rs, run `cargo uninstall cargo-embed` to uninstall your older version of cargo-embed before trying again.

## Running

For a debug build

```sh
cargo run
```

For a release build

```sh
cargo run --release
```

If you do not specify a DEFMT_LOG level, it will be set to `debug`.
That means `println!("")`, `info!("")` and `debug!("")` statements will be printed.
If you wish to override this, you can change it in `.cargo/config.toml`

```toml
[env]
DEFMT_LOG = "off"
```

You can also set this inline (on Linux/MacOS)  

```sh
DEFMT_LOG=trace cargo run
```

## Flashing CYW43 WiFI+BT module firmware

If you wish you use the on-board LED or the WiFi/BT module you will need to flash the module firmware.

`NOTE: By default firmware is placed at the 3.5MiB mark in ROM`

For WiFi + Bluetooth firmware

```sh
./flashwb.sh
```

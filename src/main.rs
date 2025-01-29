#![no_std]
#![no_main]
use defmt::*;

use rp235x_hal as hal;
use rp235x_hal::block::ImageDef;
use rp235x_hal::{binary_info, entry};

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use defmt_rtt as _;
use panic_halt as _;

// Also need a Global Allocator!!!

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::non_secure_exe();

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [binary_info::EntryAddr; 4] = [
    binary_info::rp_program_name!(c"KororaRTOS"),
    binary_info::rp_program_description!(c"The littlest penguin!"),
    binary_info::rp_cargo_version!(),
    binary_info::rp_program_build_attribute!(),
];

const WIFI_FIRMWARE_BASE: u32 = 0x1038_0000;
const BT_FIRMWARE_BASE: u32 = 0x103C_0000;
const CLM_FIRMWARE_BASE: u32 = 0x103C_4000;

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[entry]
fn main() -> ! {
    let _fw = unsafe { core::slice::from_raw_parts(WIFI_FIRMWARE_BASE as *const u8, 231077) };
    let _btfw = unsafe { core::slice::from_raw_parts(BT_FIRMWARE_BASE as *const u8, 6164) };
    let _clm = unsafe { core::slice::from_raw_parts(CLM_FIRMWARE_BASE as *const u8, 984) };

    info!("A Rusty 'Hello!' from RISC-V land!");

    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO15 as an output
    let mut led_pin = pins.gpio15.into_push_pull_output();
    loop {
        led_pin.set_high().unwrap();
        timer.delay_ms(500);
        led_pin.set_low().unwrap();
        timer.delay_ms(500);
    }
}

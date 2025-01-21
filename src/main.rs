#![no_std]
#![no_main]
use cortex_m_rt::entry;
use defmt::*;
use rp235x_hal::block::ImageDef;
use rp_binary_info as binary_info;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

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

const WIFI_FIRMWARE_BASE: u32 = 0x1030_0000;
const BT_FIRMWARE_BASE: u32 = 0x1034_0000;
const CLM_FIRMWARE_BASE: u32 = 0x1034_4000;

#[entry]
fn main() -> ! {
    let _fw = unsafe { core::slice::from_raw_parts(WIFI_FIRMWARE_BASE as *const u8, 231077) };
    let _btfw = unsafe { core::slice::from_raw_parts(BT_FIRMWARE_BASE as *const u8, 6164) };
    let _clm = unsafe { core::slice::from_raw_parts(CLM_FIRMWARE_BASE as *const u8, 984) };

    info!("Hello from 'minimal' Rust land!");
    loop {
        cortex_m::asm::nop();
    }
}

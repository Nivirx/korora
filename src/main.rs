#![no_std]
#![no_main]

use core::str;

use cyw43::ScanOptions;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};

use defmt::*;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};

use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use bt_hci::controller::ExternalController;

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Embassy Template w/ WiFi & BT"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the Pico2W w/ the embassy libraries. WiFi & BT are tested as well as the SYS_LED"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

const WIFI_FIRMWARE_BASE: u32 = 0x1030_0000;
const BT_FIRMWARE_BASE: u32 = 0x1034_0000;
const CLM_FIRMWARE_BASE: u32 = 0x1034_4000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let fw = unsafe { core::slice::from_raw_parts(WIFI_FIRMWARE_BASE as *const u8, 231077) };
    let btfw = unsafe { core::slice::from_raw_parts(BT_FIRMWARE_BASE as *const u8, 6164) };
    let clm = unsafe { core::slice::from_raw_parts(CLM_FIRMWARE_BASE as *const u8, 984) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, bt_device, mut control, runner) =
        cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // turn on LED to indicate cyw43 is 'active'
    info!("SYS_LED on");
    control.gpio_set(0, true).await;

    // prepare bluetooth as peripheral as per embassy examples
    let bt_controller: ExternalController<_, 16> = ExternalController::new(bt_device);
    let bt_peripheral_handle = ble_bas_peripheral::run::<_, 128>(bt_controller);

    // Scope wifi_scan so we can change gpio's later
    {
        let mut wifi_scan = control.scan(ScanOptions::default()).await;
        while let Some(bss) = wifi_scan.next().await {
            if let Ok(ssid_str) = str::from_utf8(&bss.ssid) {
                info!(
                    "Scanned {} == {:x}\nRSSI: {}\tPHY_NOISE: {}\tSNR: {}",
                    ssid_str, bss.bssid, bss.rssi, bss.phy_noise, bss.snr
                );
            }
        }
    }

    // 'join' our ble peripheral tasks
    bt_peripheral_handle.await;

    info!("SYS_LED off");
    control.gpio_set(0, false).await;

    let loop_delay = Duration::from_secs(10);
    let blink_delay = Duration::from_millis(125);
    loop {
        info!("All done - Waiting in loop!");
        for _ in 0..4 {
            control.gpio_set(0, true).await;
            Timer::after(blink_delay).await;
            control.gpio_set(0, false).await;
            Timer::after(blink_delay).await;
        }
        Timer::after(loop_delay).await;
    }
}

// from embassy-rs/trouble example code
mod ble_bas_peripheral {
    use defmt::*;
    use embassy_futures::{join::join, select::select};
    use embassy_time::Timer;
    use trouble_host::prelude::*;

    /// Max number of connections
    const CONNECTIONS_MAX: usize = 1;

    /// Max number of L2CAP channels.
    const L2CAP_CHANNELS_MAX: usize = 8; // Signal + att

    const MAX_ATTRIBUTES: usize = 10;

    // GATT Server definition
    #[gatt_server]
    struct Server {
        battery_service: BatteryService,
    }

    /// Battery service
    #[gatt_service(uuid = service::BATTERY)]
    struct BatteryService {
        /// Battery Level
        #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
        #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Battery Level")]
        #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 25)]
        level: u8,
        #[characteristic(uuid = "408813df-5dd4-1f87-ec11-cdb001100000", write, read, notify)]
        status: bool,
    }

    /// Run the BLE stack.
    pub async fn run<C, const L2CAP_MTU: usize>(controller: C)
    where
        C: Controller,
    {
        // Using a fixed "random" address can be useful for testing. In real scenarios, one would
        // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
        let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
        info!("Our address = {:?}", address);

        let mut resources: HostResources<C, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU> =
            HostResources::new(PacketQos::None);
        let (stack, mut peripheral, _, runner) = trouble_host::new(controller, &mut resources)
            .set_random_address(address)
            .build();

        info!("Starting advertising and GATT service");
        let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
            name: "TrouBLE",
            appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
        }))
        .unwrap();

        let _ = join(ble_task(runner), async {
            loop {
                match advertise("Trouble Example", &mut peripheral).await {
                    Ok(conn) => {
                        // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                        let a = gatt_events_task(&server, &conn);
                        let b = custom_task(&server, &conn, stack);
                        // run until any task ends (usually because the connection has been closed),
                        // then quit the ble demo.
                        select(a, b).await;
                        break;
                    }
                    Err(e) => {
                        let e = defmt::Debug2Format(&e);
                        self::panic!("[adv] error: {:?}", e);
                    }
                }
            }
        })
        .await;
    }

    /// This is a background task that is required to run forever alongside any other BLE tasks.
    ///
    /// ## Alternative
    ///
    /// If you didn't require this to be generic for your application, you could statically spawn this with i.e.
    ///
    /// ```rust [ignore]
    ///
    /// #[embassy_executor::task]
    /// async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>>) {
    ///     runner.run().await;
    /// }
    ///
    /// spawner.must_spawn(ble_task(runner));
    /// ```
    async fn ble_task<C: Controller>(mut runner: Runner<'_, C>) {
        loop {
            if let Err(e) = runner.run().await {
                let e = defmt::Debug2Format(&e);
                self::panic!("[ble_task] error: {:?}", e);
            }
        }
    }

    /// Stream Events until the connection closes.
    ///
    /// This function will handle the GATT events and process them.
    /// This is how we interact with read and write requests.
    async fn gatt_events_task(server: &Server<'_>, conn: &Connection<'_>) -> Result<(), Error> {
        let level = server.battery_service.level;
        loop {
            match conn.next().await {
                ConnectionEvent::Disconnected { reason } => {
                    info!("[gatt] disconnected: {:?}", reason);
                    break;
                }
                ConnectionEvent::Gatt { data } => {
                    // We can choose to handle event directly without an attribute table
                    // let req = data.request();
                    // ..
                    // data.reply(conn, Ok(AttRsp::Error { .. }))

                    // But to simplify things, process it in the GATT server that handles
                    // the protocol details
                    match data.process(server).await {
                        // Server processing emits
                        Ok(Some(GattEvent::Read(event))) => {
                            if event.handle() == level.handle {
                                let value = server.get(&level);
                                info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                            }
                        }
                        Ok(Some(GattEvent::Write(event))) => {
                            if event.handle() == level.handle {
                                info!(
                                    "[gatt] Write Event to Level Characteristic: {:?}",
                                    event.data()
                                );
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("[gatt] error processing event: {:?}", e);
                        }
                    }
                }
            }
        }
        info!("[gatt] task finished");
        Ok(())
    }

    /// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
    async fn advertise<'a, C: Controller>(
        name: &'a str,
        peripheral: &mut Peripheral<'a, C>,
    ) -> Result<Connection<'a>, BleHostError<C::Error>> {
        let mut advertiser_data = [0; 31];
        AdStructure::encode_slice(
            &[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16([0x0f, 0x18])]),
                AdStructure::CompleteLocalName(name.as_bytes()),
            ],
            &mut advertiser_data[..],
        )?;
        let advertiser = peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &advertiser_data[..],
                    scan_data: &[],
                },
            )
            .await?;
        info!("[adv] advertising");
        let conn = advertiser.accept().await?;
        info!("[adv] connection established");
        Ok(conn)
    }

    /// Example task to use the BLE notifier interface.
    /// This task will notify the connected central of a counter value every 2 seconds.
    /// It will also read the RSSI value every 2 seconds.
    /// and will stop when the connection is closed by the central or an error occurs.
    async fn custom_task<C: Controller>(
        server: &Server<'_>,
        conn: &Connection<'_>,
        stack: Stack<'_, C>,
    ) {
        let mut tick: u8 = 0;
        let level = server.battery_service.level;
        loop {
            tick = tick.wrapping_add(1);
            info!("[custom_task] notifying connection of tick {}", tick);
            if level.notify(server, conn, &tick).await.is_err() {
                info!("[custom_task] error notifying connection");
                break;
            };
            // read RSSI (Received Signal Strength Indicator) of the connection.
            if let Ok(rssi) = conn.rssi(stack).await {
                info!("[custom_task] RSSI: {:?}", rssi);
            } else {
                info!("[custom_task] error getting RSSI");
                break;
            };
            Timer::after_secs(2).await;
        }
    }
}

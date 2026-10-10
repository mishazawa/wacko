use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::RNG;
use embassy_nrf::{bind_interrupts, rng};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, SoftdeviceController, mpsl};
use static_cell::StaticCell;
use trouble_host::advertise::{AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE};
use trouble_host::prelude::*;
use trouble_host::{Address, HostResources};

// mod gap;
mod gatt;
mod server;

// pub use gap::advertise;
pub use server::BleServer;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

pub const MSP_NORDIC_COMPANY_ID: u16 = 0x0059;
pub const MSP_PAYLOAD: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

const BLE_NAME: &str = "misha_";
const L2CAP_MTU: usize = 251;
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att
const L2CAP_TXQ: u8 = 3;

/// How many incoming L2CAP buffers per link
const L2CAP_RXQ: u8 = 3;
pub type BleController = SoftdeviceController<'static>;

pub type BleResources =
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, BleController, DefaultPacketPool>) {
    runner.run().await.expect("Can't spawn ble task")
}

pub async fn init(
    spawner: Spawner,
) -> Result<
    (
        Peripheral<'static, BleController, DefaultPacketPool>,
        BleServer<'static>,
    ),
    nrf_sdc::Error,
> {
    let p = embassy_nrf::init(Default::default());
    let mpsl_p =
        mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };

    let mpsl = {
        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        MPSL.init(mpsl::MultiprotocolServiceLayer::new(mpsl_p, Irqs, lfclk_cfg).unwrap())
    };

    let sdc_p: nrf_sdc::Peripherals<'_> = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );

    let rng = {
        static CELL: StaticCell<rng::Rng<Async>> = StaticCell::new();
        CELL.init(rng::Rng::new(p.RNG, Irqs))
    };
    let sdc_mem = {
        static CELL: StaticCell<sdc::Mem<4696>> = StaticCell::new();
        CELL.init(sdc::Mem::<4696>::new())
    };

    let sdc = build_sdc(sdc_p, rng, mpsl, sdc_mem)?;
    let address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let resources = {
        static CELL: StaticCell<BleResources> = StaticCell::new();
        CELL.init(HostResources::new())
    };

    let stack = {
        static CELL: StaticCell<Stack<'static, BleController, DefaultPacketPool>> =
            StaticCell::new();
        CELL.init(
            trouble_host::new(sdc, resources)
                .set_random_address(address)
                .build(),
        )
    };

    let runner = stack.runner();

    let server = BleServer::create();

    spawner.must_spawn(mpsl_task(mpsl));
    spawner.must_spawn(ble_task(runner));

    let peripheral = stack.peripheral();
    Ok((peripheral, server))
}

fn build_sdc<const N: usize>(
    p: nrf_sdc::Peripherals<'static>,
    rng: &'static mut rng::Rng<Async>,
    mpsl: &'static MultiprotocolServiceLayer,
    mem: &'static mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'static>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}
pub async fn advertise<'a, 'b>(
    peripheral: &mut Peripheral<'a, BleController, DefaultPacketPool>,
    server: &'b BleServer<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<Error>> {
    const GAP_ADV_LIMIT: usize = 31;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];

    let ad_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED), // flags
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),                // name
            AdStructure::Unknown {
                ty: 0x19,
                data: &[0xc1, 0x03],
            }, // kbd
            AdStructure::CompleteServiceUuids16(&[[0x12, 0x18]]),               // hid
        ],
        &mut advertiser_data[..],
    )
    .expect("Can't create advertise structure");
    let mut scan_resp = [0; GAP_ADV_LIMIT];

    let sr_len = AdStructure::encode_slice(
        &[
            AdStructure::ManufacturerSpecificData {
                company_identifier: MSP_NORDIC_COMPANY_ID,
                payload: &MSP_PAYLOAD,
            },
            AdStructure::CompleteServiceUuids16(&[
                service::DEVICE_INFORMATION.to_le_bytes(),
                service::BATTERY.to_le_bytes(),
                service::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                service::GATT.to_le_bytes(),
            ]),
        ],
        &mut scan_resp[..],
    )
    .expect("Can't create scan response structure");

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[0..ad_len],
                scan_data: &scan_resp[0..sr_len],
            },
        )
        .await
        .expect("Can't create advertiser.");

    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    Ok(conn)
}

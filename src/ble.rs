use defmt::info;
use embassy_futures::select;

use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use trouble_host::prelude::*;

#[gatt_server]
struct Server {
    shutter_service: ShutterService,
}
#[gatt_service(uuid= service::BATTERY)]
struct ShutterService {
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify)]
    level: u8,
}

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) {
    mpsl.run().await
}

const BLE_NAME: &str = "misha_";
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;
const L2CAP_MTU: usize = 251;

type BleResources =
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

type BLERunner = Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>;

pub async fn run<C>(sdc: C)
where
    C: Controller,
{
    let address = Address::random([0x42, 0x6A, 0xE3, 0x1E, 0x83, 0xE7]);
    info!("Our address = {:?}", address);

    let mut resources = BleResources::new();

    let stack = trouble_host::new(sdc, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: BLE_NAME,
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .expect("Failed to create GATT server");

    let app_task = async {
        loop {
            match advertise(&mut peripheral, &server).await {
                Ok(conn) => {
                    //
                }
                Err(e) => {
                    let e = defmt::Debug2Format(&e);
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    };
    select::select(ble_task(runner), app_task).await;
}

async fn advertise<'a, 'b, C: Controller>(
    peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<C::Error>> {
    const GAP_ADV_LIMIT: usize = 31;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];

    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[service::BATTERY.to_le_bytes()]),
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[0..len],
                scan_data: &[],
            },
        )
        .await?;

    info!("Starting advertise!");

    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("Connection established!");

    Ok(conn)
}

async fn ble_task<C: Controller, P: PacketPool>(
    mut runner: Runner<'_, C, P>,
) -> Result<(), BleHostError<C::Error>> {
    runner.run().await
}

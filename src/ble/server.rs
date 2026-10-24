use defmt::info;
use defmt::warn;
use embassy_executor::Spawner;
use microbit_bsp::ble::MultiprotocolServiceLayer;
use microbit_bsp::ble::SoftdeviceController;
use microbit_bsp::ble::SoftdeviceError;
use static_cell::StaticCell;
use trouble_host::prelude::*;

use crate::ble::BT_NAME;
use crate::btn::BTN_EVENTS;
use crate::btn::BtnEvent;

use super::constants::*;
use super::gatt::*;

const L2CAP_MTU: usize = 251;

pub type BleController = SoftdeviceController<'static>;

pub type BleResources =
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[gatt_server]
pub struct Server {
    pub hid: HidService,
    bas: BatteryService,
    dis: DeviceInformationService,
}

impl Server<'static> {
    pub fn start_gatt(
        spawner: Spawner,
        controller: BleController,
        mpsl: &'static MultiprotocolServiceLayer<'_>,
    ) -> Result<
        (
            &'static Self,
            Peripheral<'static, BleController, DefaultPacketPool>,
        ),
        BleHostError<SoftdeviceError>,
    > {
        spawner.must_spawn(mpsl_task(mpsl));

        let address = Address::random([0x42, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let resources = {
            static RESOURCES: StaticCell<BleResources> = StaticCell::new();
            RESOURCES.init(BleResources::new())
        };
        let stack = {
            static STACK: StaticCell<Stack<'static, BleController, DefaultPacketPool>> =
                StaticCell::new();
            STACK.init(
                trouble_host::new(controller, resources)
                    .set_random_address(address)
                    .set_io_capabilities(IoCapabilities::NoInputNoOutput)
                    .build(),
            )
        };

        let runner = stack.runner();
        let peripheral = stack.peripheral();

        let server = {
            static SERVER: StaticCell<Server<'_>> = StaticCell::new();
            SERVER.init(
                Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
                    name: BT_NAME,
                    appearance: &appearance::human_interface_device::KEYBOARD,
                }))
                .expect("Error creating Gatt Server"),
            )
        };
        info!("Starting Gatt Server");
        spawner.must_spawn(ble_task(runner));
        Ok((server, peripheral))
    }

    pub async fn advertise<'a>(
        peripheral: &mut Peripheral<'static, BleController, DefaultPacketPool>,
        server: &'a Server<'static>,
    ) -> Result<GattConnection<'static, 'a, DefaultPacketPool>, BleHostError<SoftdeviceError>> {
        let mut adv = [0u8; 31];
        let adv_len = AdStructure::encode_slice(
            &[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::IncompleteServiceUuids16(&[[0x0F, 0x18], [0x12, 0x18]]),
                AdStructure::CompleteLocalName(BT_NAME.as_bytes()),
                AdStructure::Unknown {
                    ty: 0x19,
                    data: &[0xC1, 0x03],
                }, // appearance: keyboard
            ],
            &mut adv[..],
        )?;

        let mut scan = [0u8; 31];
        let scan_len = AdStructure::encode_slice(
            &[AdStructure::CompleteServiceUuids16(&[
                [0x0A, 0x18],
                [0x0F, 0x18],
                [0x12, 0x18],
            ])],
            &mut scan[..],
        )?;

        let advertiser = peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv[..adv_len],
                    scan_data: &scan[..scan_len],
                },
            )
            .await?;

        let conn = advertiser.accept().await?.with_attribute_server(server)?;

        let _ = conn.raw().set_bondable(true);
        if let Err(e) = conn.raw().request_security() {
            warn!("request_security: {:?}", e); // error if already encrypted; fine
        }
        Ok(conn)
    }

    pub async fn key_task(
        server: &Server<'static>,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
    ) {
        loop {
            let report = match BTN_EVENTS.receive().await {
                BtnEvent::Pressed => [0u8, 0, 0x04, 0, 0, 0, 0, 0], // 'a' down
                BtnEvent::Released => [0u8; 8],                     // all keys up
            };
            if server
                .hid
                .input_keyboard
                .notify(conn, &report, true)
                .await
                .is_err()
            {
                break; // not subscribed or disconnected
            }
        }
    }

    pub async fn gatt_events(conn: &GattConnection<'_, '_, DefaultPacketPool>) {
        loop {
            match conn.next().await {
                GattConnectionEvent::Disconnected { reason } => {
                    info!("disconnected: {:?}", reason);
                    break;
                }
                GattConnectionEvent::Gatt { event } => match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("gatt reply error: {:?}", e),
                },
                _ => {}
            }
        }
    }
}

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, BleController, DefaultPacketPool>) {
    runner.run().await.expect("Error in BLE task");
}

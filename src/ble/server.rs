use defmt::info;
use embassy_executor::Spawner;
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController, SoftdeviceError};
use static_cell::StaticCell;
use trouble_host::prelude::*;

use crate::ble::{BLE_NAME, BleController, BleResources, ble_task, gatt::ButtonService, mpsl_task};

#[gatt_server]
pub struct BleServer {
    pub hid: ButtonService,
}

impl<'d> BleServer<'d> {
    pub fn start_gatt(
        spawner: Spawner,
        controller: BleController,
        mpsl: &'static MultiprotocolServiceLayer<'_>,
    ) -> Result<
        (
            &'static Self,
            Peripheral<'d, BleController, DefaultPacketPool>,
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
            static STACK: StaticCell<Stack<'_, SoftdeviceController<'_>, DefaultPacketPool>> =
                StaticCell::new();
            STACK.init(trouble_host::new(controller, resources).set_random_address(address))
        };

        let host = stack.build();

        let server = {
            static SERVER: StaticCell<BleServer<'_>> = StaticCell::new();
            SERVER.init(
                BleServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
                    name: BLE_NAME,
                    appearance: &appearance::human_interface_device::KEYBOARD,
                }))
                .expect("Error creating Gatt Server"),
            )
        };
        info!("Starting Gatt Server");
        spawner.must_spawn(ble_task(host.runner));
        Ok((server, host.peripheral))
    }

    pub async fn gatt_task(&self, conn: &GattConnection<'_, '_, DefaultPacketPool>) {
        loop {
            match conn.next().await {
                GattConnectionEvent::Disconnected { reason } => {
                    info!("[gatt] Disconnected: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }
    }
}

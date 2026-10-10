use defmt::info;
use trouble_host::prelude::*;

use crate::ble::{
    BLE_NAME,
    gatt::{BatteryInfoService, DeviceInfoService, GattService, HidService},
};

#[gatt_server]
pub struct BleServer {
    pub hid_info: HidService,
    device_info: DeviceInfoService,
    battery: BatteryInfoService,
    gatt: GattService,
}

impl<'d> BleServer<'d> {
    pub fn create() -> Self {
        BleServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
            name: BLE_NAME,
            appearance: &appearance::human_interface_device::KEYBOARD,
        }))
        .expect("Error create gatt server")
    }

    pub async fn gatt_task(&self, conn: &GattConnection<'_, '_, DefaultPacketPool>) {
        loop {
            match conn.next().await {
                // Ga
                GattConnectionEvent::Disconnected { reason } => {
                    info!("[gatt] Disconnected: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }
    }
}

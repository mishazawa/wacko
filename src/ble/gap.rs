use nrf_softdevice::{
    Softdevice,
    ble::{
        Connection,
        advertisement_builder::*,
        peripheral::{self, AdvertiseError},
    },
};

use crate::ble::gatt::HidSecurityHandler;

pub static ADV_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
    .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
    .services_16(
        ServiceList::Incomplete,
        &[
            ServiceUuid16::BATTERY,
            ServiceUuid16::HUMAN_INTERFACE_DEVICE,
        ],
    )
    .full_name("HelloRust")
    // Change the appearance (icon of the bluetooth device) to a keyboard
    .raw(AdvertisementDataType::APPEARANCE, &[0xC1, 0x03])
    .build();

pub static SCAN_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
    .services_16(
        ServiceList::Complete,
        &[
            ServiceUuid16::DEVICE_INFORMATION,
            ServiceUuid16::BATTERY,
            ServiceUuid16::HUMAN_INTERFACE_DEVICE,
        ],
    )
    .build();

pub static SEC: HidSecurityHandler = HidSecurityHandler {};

pub async fn advertise(sd: &'static Softdevice) -> Result<Connection, AdvertiseError> {
    let config = peripheral::Config::default();
    let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
        adv_data: &ADV_DATA,
        scan_data: &SCAN_DATA,
    };
    peripheral::advertise_pairable(sd, adv, &config, &SEC).await
}

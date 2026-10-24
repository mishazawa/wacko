use trouble_host::prelude::*;

use crate::ble::{MANUFACTURER_NAME_STRING, MODEL_NUMBER_DATA, SERIAL_NUMBER, constants::*};

#[gatt_service(uuid = service::DEVICE_INFORMATION)]
pub struct DeviceInformationService {
    #[characteristic(uuid = characteristic::PNP_ID, read)]
    pub(crate) pnp_id: PnPID,
    #[characteristic(uuid = characteristic::MANUFACTURER_NAME_STRING, read, value = MANUFACTURER_NAME_STRING)]
    manufacturer_name: &'static str,
    #[characteristic(uuid = characteristic::MODEL_NUMBER_STRING, read, value = MODEL_NUMBER_DATA)]
    model_number: &'static str,
    #[characteristic(uuid = characteristic::SERIAL_NUMBER_STRING, read, value = SERIAL_NUMBER)]
    serial_number: &'static str,
}

#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub struct HidService {
    #[characteristic(uuid = characteristic::HID_INFORMATION, read, value = [0x01, 0x01, 0x00, 0x03])]
    hid_info: [u8; 4],
    #[characteristic(uuid = characteristic::REPORT_MAP, read, value = HID_REPORT_DESCRIPTOR)]
    report_map: [u8; 127],
    #[characteristic(uuid = characteristic::HID_CONTROL_POINT, write_without_response)]
    hid_control: [u8; 1],
    #[characteristic(uuid = characteristic::PROTOCOL_MODE, read, write_without_response, value=[1u8])]
    protocol_mode: [u8; 1],
    #[descriptor(uuid = descriptors::REPORT_REFERENCE, read, value = [KEYBOARD_ID, 1u8])]
    #[characteristic(uuid = characteristic::REPORT, read, notify, value = [0u8; 8])]
    pub input_keyboard: [u8; 8],
    #[descriptor(uuid = descriptors::REPORT_REFERENCE, read, value = [KEYBOARD_ID, 2u8])]
    #[characteristic(uuid = characteristic::REPORT, read, value = [0u8; 8])]
    output_keyboard: [u8; 8],
    #[descriptor(uuid = descriptors::REPORT_REFERENCE, read, value = [MEDIA_KEYS_ID, 1u8])]
    #[characteristic(uuid = characteristic::REPORT, read, value = [0u8; 2])]
    media_keys: [u8; 2],
}

#[gatt_service(uuid = service::BATTERY)]
pub struct BatteryService {
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Battery Level")]
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify)]
    level: u8,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum VidSource {
    BluetoothSIG = 1,
    UsbIF = 2,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PnPID {
    pub(crate) vid_source: VidSource,
    pub(crate) vendor_id: u16,
    pub(crate) product_id: u16,
    pub(crate) product_version: u16,
}

impl AsGatt for PnPID {
    const MIN_SIZE: usize = core::mem::size_of::<PnPID>();

    const MAX_SIZE: usize = core::mem::size_of::<PnPID>();

    fn as_gatt(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<PnPID>(),
            )
        }
    }
}

impl Default for PnPID {
    fn default() -> Self {
        Self {
            vid_source: VidSource::UsbIF,
            vendor_id: 0xDEAD,
            product_id: 0xBEEF,
            product_version: 0x0000,
        }
    }
}

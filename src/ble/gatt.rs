use trouble_host::prelude::{
    characteristic::SERVICE_CHANGED,
    service::{BATTERY, DEVICE_INFORMATION, GATT},
    service_class::HID,
    *,
};

macro_rules! count {
	() => { 0u8 };
	($x:tt $($xs:tt)*) => {1u8 + count!($($xs)*)};
}

macro_rules! hid {
	($(( $($xs:tt),*)),+ $(,)?) => { [ $( (count!($($xs)*)-1) | $($xs),* ),* ] };
}

// Main items
const HIDINPUT: u8 = 0x80;
const HIDOUTPUT: u8 = 0x90;
const COLLECTION: u8 = 0xa0;
const END_COLLECTION: u8 = 0xc0;

// Global items
const USAGE_PAGE: u8 = 0x04;
const LOGICAL_MINIMUM: u8 = 0x14;
const LOGICAL_MAXIMUM: u8 = 0x24;
const REPORT_SIZE: u8 = 0x74; //bits
const REPORT_ID: u8 = 0x84;
const REPORT_COUNT: u8 = 0x94; //bytes

// Local items
const USAGE: u8 = 0x08;
const USAGE_MINIMUM: u8 = 0x18;
const USAGE_MAXIMUM: u8 = 0x28;

const KEYBOARD_ID: u8 = 0x01;

const REPOST_DESC: [u8; 65] = hid!(
    (USAGE_PAGE, 0x01), // USAGE_PAGE (Generic Desktop Ctrls)
    (USAGE, 0x06),      // USAGE (Keyboard)
    (COLLECTION, 0x01), // COLLECTION (Application)
    // ------------------------------------------------- Keyboard
    (REPORT_ID, KEYBOARD_ID), //   REPORT_ID (1)
    (USAGE_PAGE, 0x07),       //   USAGE_PAGE (Kbrd/Keypad)
    (USAGE_MINIMUM, 0xE0),    //   USAGE_MINIMUM (0xE0)
    (USAGE_MAXIMUM, 0xE7),    //   USAGE_MAXIMUM (0xE7)
    (LOGICAL_MINIMUM, 0x00),  //   LOGICAL_MINIMUM (0)
    (LOGICAL_MAXIMUM, 0x01),  //   Logical Maximum (1)
    (REPORT_SIZE, 0x01),      //   REPORT_SIZE (1)
    (REPORT_COUNT, 0x08),     //   REPORT_COUNT (8)
    (HIDINPUT, 0x02), //   INPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 1 byte (Reserved)
    (REPORT_SIZE, 0x08), //   REPORT_SIZE (8)
    (HIDINPUT, 0x01), //   INPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (REPORT_COUNT, 0x05), //   REPORT_COUNT (5) ; 5 bits (Num lock, Caps lock, Scroll lock, Compose, Kana)
    (REPORT_SIZE, 0x01),  //   REPORT_SIZE (1)
    (USAGE_PAGE, 0x08),   //   USAGE_PAGE (LEDs)
    (USAGE_MINIMUM, 0x01), //   USAGE_MINIMUM (0x01) ; Num Lock
    (USAGE_MAXIMUM, 0x05), //   USAGE_MAXIMUM (0x05) ; Kana
    (HIDOUTPUT, 0x02), //   OUTPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 3 bits (Padding)
    (REPORT_SIZE, 0x03), //   REPORT_SIZE (3)
    (HIDOUTPUT, 0x01), //   OUTPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    (REPORT_COUNT, 0x06), //   REPORT_COUNT (6) ; 6 bytes (Keys)
    (REPORT_SIZE, 0x08), //   REPORT_SIZE(8)
    (LOGICAL_MINIMUM, 0x00), //   LOGICAL_MINIMUM(0)
    (LOGICAL_MAXIMUM, 0x65), //   LOGICAL_MAXIMUM(0x65) ; 101 keys
    (USAGE_PAGE, 0x07), //   USAGE_PAGE (Kbrd/Keypad)
    (USAGE_MINIMUM, 0x00), //   USAGE_MINIMUM (0)
    (USAGE_MAXIMUM, 0x65), //   USAGE_MAXIMUM (0x65)
    (HIDINPUT, 0x00),  //   INPUT (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (END_COLLECTION),  // END_COLLECTION
);

// SDP
#[gatt_service(uuid = HID)]
pub struct HidService {
    /* Information Characteristic */
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    hid_info: [u8; 4],
    // BLE HID Report Map characteristic
    #[characteristic(uuid = "2a4b", read, value = REPOST_DESC)]
    report_map: [u8; 65],
    // BLE HID control point characteristic
    #[characteristic(uuid = "2a4c", write_without_response)]
    hid_control_point: u8,
    // BLE HID protocol mode characteristic
    #[characteristic(uuid = "2a4e", read, write_without_response, value = 1)]
    protocol_mode: u8,
    // BLE HID Report characteristic
    #[descriptor(uuid = "2908", read, value=[KEYBOARD_ID, 1])]
    #[characteristic(uuid = "2a4d", read, notify, value=[0,0,0,0,0,0,0,0])]
    pub input_keyboard: [u8; 8],
    #[descriptor(uuid = "2908", read, value=[KEYBOARD_ID, 2])]
    #[characteristic(uuid = "2a4d", read, write, write_without_response)]
    pub output_keyboard: [u8; 1],
}

#[gatt_service(uuid = DEVICE_INFORMATION)]
pub struct DeviceInfoService {
    #[characteristic(uuid = "2a50", read, value = [0x02, 0x8a, 0x24, 0x66, 0x82, 0x34, 0x36])]
    device_info: [u8; 7],
}

#[gatt_service(uuid = BATTERY)]
pub struct BatteryInfoService {
    #[characteristic(uuid = "2a19", notify, value = 100)]
    level: u8,
}

#[gatt_service(uuid = GATT)]
pub struct GattService {
    #[characteristic(uuid = SERVICE_CHANGED, read, notify, value=[1, 0, 255, 255])]
    val: [u8; 4],
}

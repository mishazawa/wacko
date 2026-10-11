use defmt::info;
use nrf_softdevice::{
    Softdevice,
    ble::{
        Connection, Uuid,
        gatt_server::{self, CharacteristicHandles, RegisterError, builder::ServiceBuilder},
        security::SecurityHandler,
    },
};

use crate::ble::constants::*;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};

pub struct DeviceInformationService {}
pub struct BatteryService {
    value_handle: u16,
    cccd_handle: u16,
}
#[allow(dead_code)]
pub struct HidService {
    hid_info: u16,
    report_map: u16,
    hid_control: u16,
    protocol_mode: u16,
    input_keyboard: u16,
    input_keyboard_cccd: u16,
    input_keyboard_descriptor: u16,
    output_keyboard: u16,
    output_keyboard_descriptor: u16,
    input_media_keys: u16,
    input_media_keys_cccd: u16,
    input_media_keys_descriptor: u16,
}

impl HidService {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service_builder = ServiceBuilder::new(sd, Uuid::new_16(0x1812))?;
        let hid_info = service_builder.add_characteristic(
            HID_INFO,
            Attribute::new([0x11u8, 0x1u8, 0x00u8, 0x01u8]),
            Metadata::new(Properties::new().read()),
        )?;
        let hid_info_handle = hid_info.build();

        let report_map = service_builder.add_characteristic(
            REPORT_MAP,
            Attribute::new(HID_REPORT_DESCRIPTOR),
            Metadata::new(Properties::new().read()),
        )?;
        let report_map_handle = report_map.build();

        let hid_control = service_builder.add_characteristic(
            HID_CONTROL_POINT,
            Attribute::new([0u8]),
            Metadata::new(Properties::new().write_without_response()),
        )?;
        let hid_control_handle = hid_control.build();

        let protocol_mode = service_builder.add_characteristic(
            PROTOCOL_MODE,
            Attribute::new([1u8]),
            Metadata::new(Properties::new().read().write_without_response()),
        )?;
        let protocol_mode_handle = protocol_mode.build();

        let mut input_keyboard = service_builder.add_characteristic(
            HID_REPORT,
            Attribute::new([0u8; 8]),
            Metadata::new(Properties::new().read().notify()),
        )?;
        let input_keyboard_desc = input_keyboard
            .add_descriptor(Uuid::new_16(0x2908), Attribute::new([KEYBOARD_ID, 1u8]))?;
        let input_keyboard_handle = input_keyboard.build();

        let mut output_keyboard = service_builder.add_characteristic(
            HID_REPORT,
            Attribute::new([0u8; 8]),
            Metadata::new(Properties::new().read().write().write_without_response()),
        )?;
        let output_keyboard_desc = output_keyboard
            .add_descriptor(Uuid::new_16(0x2908), Attribute::new([KEYBOARD_ID, 2u8]))?;
        let output_keyboard_handle = output_keyboard.build();

        let mut input_media_keys = service_builder.add_characteristic(
            HID_REPORT,
            Attribute::new([0u8; 16]),
            Metadata::new(Properties::new().read().notify()),
        )?;
        let input_media_keys_desc = input_media_keys
            .add_descriptor(Uuid::new_16(0x2908), Attribute::new([MEDIA_KEYS_ID, 1u8]))?;
        let input_media_keys_handle = input_media_keys.build();

        let _service_handle = service_builder.build();

        Ok(Self {
            hid_info: hid_info_handle.value_handle,
            report_map: report_map_handle.value_handle,
            hid_control: hid_control_handle.value_handle,
            protocol_mode: protocol_mode_handle.value_handle,
            input_keyboard: input_keyboard_handle.value_handle,
            input_keyboard_cccd: input_keyboard_handle.cccd_handle,
            input_keyboard_descriptor: input_keyboard_desc.handle(),
            output_keyboard: output_keyboard_handle.value_handle,
            output_keyboard_descriptor: output_keyboard_desc.handle(),
            input_media_keys: input_media_keys_handle.value_handle,
            input_media_keys_cccd: input_media_keys_handle.cccd_handle,
            input_media_keys_descriptor: input_media_keys_desc.handle(),
        })
    }

    pub(crate) fn on_write(
        &self,
        conn: &nrf_softdevice::ble::Connection,
        handle: u16,
        data: &[u8],
    ) {
        if handle == self.output_keyboard {
            info!("HID output keyboard: {:?}", data);

            let res = if *data.get(0).unwrap() == 1 {
                [
                    0, // Modifiers (Shift, Ctrl, Alt, GUI, etc.)
                    0, // Reserved
                    0x0E, 0, 0, 0, 0,
                    0, // Key code array - 0x04 is 'a' and 0x1d is 'z' - for example
                ]
            } else {
                // kbd report clear?
                [0u8; 8]
            };
            gatt_server::notify_value(conn, self.input_keyboard, &res).unwrap();
        }
    }
}

impl BatteryService {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service_builder = ServiceBuilder::new(sd, BATTERY_SERVICE)?;

        let batt_info = service_builder.add_characteristic(
            BATTERY_LEVEL,
            Attribute::new(&[0u8]),
            Metadata::new(Properties::new().read().notify()),
        )?;

        let batt_info_handle = batt_info.build();
        let _service_handle = service_builder.build();

        Ok(Self {
            value_handle: batt_info_handle.value_handle,
            cccd_handle: batt_info_handle.cccd_handle,
        })
    }

    pub(crate) fn on_write(&self, handle: u16, data: &[u8]) -> () {
        if handle == self.cccd_handle && !data.is_empty() {
            info!("battery notifications: {}", (data[0] & 0x01) != 0);
        }
    }

    pub fn battery_level_set(
        &self,
        sd: &Softdevice,
        val: u8,
    ) -> Result<(), gatt_server::SetValueError> {
        gatt_server::set_value(sd, self.value_handle, &[val])
    }

    pub fn battery_level_notify(
        &self,
        conn: &Connection,
        val: u8,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(conn, self.value_handle, &[val])
    }

    pub fn battery_level_get(&self, sd: &Softdevice) -> Result<u8, gatt_server::GetValueError> {
        let buf = &mut [0u8];
        gatt_server::get_value(sd, self.value_handle, buf)?;
        Ok(buf[0])
    }
}

impl DeviceInformationService {
    pub fn new(
        sd: &mut Softdevice,
        pnp_id: &PnPID,
        info: DeviceInformation,
    ) -> Result<Self, RegisterError> {
        let mut sb = ServiceBuilder::new(sd, DEVICE_INFORMATION)?;

        Self::add_pnp_characteristic(&mut sb, pnp_id)?;
        Self::add_opt_str_characteristic(&mut sb, MANUFACTURER_NAME, info.manufacturer_name)?;
        Self::add_opt_str_characteristic(&mut sb, MODEL_NUMBER, info.model_number)?;
        Self::add_opt_str_characteristic(&mut sb, SERIAL_NUMBER, info.serial_number)?;
        Self::add_opt_str_characteristic(&mut sb, HARDWARE_REVISION, info.hw_rev)?;
        Self::add_opt_str_characteristic(&mut sb, FIRMWARE_REVISION, info.fw_rev)?;
        Self::add_opt_str_characteristic(&mut sb, SOFTWARE_REVISION, info.sw_rev)?;

        let _service_handle = sb.build();

        Ok(DeviceInformationService {})
    }

    fn add_opt_str_characteristic(
        sb: &mut ServiceBuilder,
        uuid: Uuid,
        val: Option<&'static str>,
    ) -> Result<Option<CharacteristicHandles>, RegisterError> {
        if let Some(val) = val {
            let attr = Attribute::new(val);
            let md = Metadata::new(Properties::new().read());
            Ok(Some(sb.add_characteristic(uuid, attr, md)?.build()))
        } else {
            Ok(None)
        }
    }

    fn add_pnp_characteristic(
        sb: &mut ServiceBuilder,
        pnp_id: &PnPID,
    ) -> Result<CharacteristicHandles, RegisterError> {
        // SAFETY: `PnPID` is `repr(C, packed)` so viewing it as an immutable slice of bytes is safe.
        let val = unsafe {
            core::slice::from_raw_parts(
                pnp_id as *const _ as *const u8,
                core::mem::size_of::<PnPID>(),
            )
        };

        let attr = Attribute::new(val);
        let md = Metadata::new(Properties::new().read());
        Ok(sb.add_characteristic(PNP_ID, attr, md)?.build())
    }
}
pub struct HidSecurityHandler {}

impl SecurityHandler for HidSecurityHandler {}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum VidSource {
    BluetoothSIG = 1,
    UsbIF = 2,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PnPID {
    pub vid_source: VidSource,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
}

#[derive(Debug, Default, defmt::Format)]
pub struct DeviceInformation {
    pub manufacturer_name: Option<&'static str>,
    pub model_number: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub hw_rev: Option<&'static str>,
    pub fw_rev: Option<&'static str>,
    pub sw_rev: Option<&'static str>,
}

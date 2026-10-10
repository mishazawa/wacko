use nrf_softdevice::{
    Softdevice,
    ble::gatt_server::{self, RegisterError},
};

use crate::ble::{
    SERIAL_NUMBER,
    gatt::{
        BatteryService, DeviceInformation, DeviceInformationService, HidService, PnPID, VidSource,
    },
};

pub struct Server {
    _dis: DeviceInformationService,
    bas: BatteryService,
    hid: HidService,
}

impl Server {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let dis = DeviceInformationService::new(
            sd,
            &PnPID {
                vid_source: VidSource::UsbIF,
                vendor_id: 0xDEAD,
                product_id: 0xBEEF,
                product_version: 0x0000,
            },
            DeviceInformation {
                manufacturer_name: Some("Embassy"),
                model_number: Some("M1234"),
                serial_number: Some(SERIAL_NUMBER),
                ..Default::default()
            },
        )?;

        let bas = BatteryService::new(sd)?;

        let hid = HidService::new(sd)?;

        Ok(Self {
            _dis: dis,
            bas,
            hid,
        })
    }
}

impl gatt_server::Server for Server {
    type Event = ();

    fn on_write(
        &self,
        conn: &nrf_softdevice::ble::Connection,
        handle: u16,
        op: gatt_server::WriteOp,
        offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        todo!()
    }
}

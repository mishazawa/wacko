use defmt::info;
use embassy_executor::Spawner;
use nrf_softdevice::{
    Softdevice,
    ble::{
        Connection,
        gatt_server::{self, RegisterError},
        peripheral::AdvertiseError,
    },
};

use crate::ble::{
    SERIAL_NUMBER,
    gap::{advertise, init_sd},
    gatt::{
        BatteryService, DeviceInformation, DeviceInformationService, HidService, PnPID, VidSource,
    },
};

pub struct Server {
    sd: &'static Softdevice,
    _dis: DeviceInformationService,
    bas: BatteryService,
    pub(crate) hid: HidService,
}

impl Server {
    pub fn new(spawner: Spawner) -> Result<Self, RegisterError> {
        let sd = init_sd()?;

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

        spawner.must_spawn(softdevice_task(sd));

        Ok(Self {
            sd,
            _dis: dis,
            bas,
            hid,
        })
    }

    pub async fn advertise(&self) -> Result<Connection, AdvertiseError> {
        advertise(self.sd).await
    }

    pub async fn on_connection(&self, conn: &Connection) {
        let e = gatt_server::run(&conn, self, |_| {}).await;
        info!("gatt_server run exited with error: {:?}", e);
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
        self.hid.on_write(conn, handle, data);
        self.bas.on_write(handle, data);
        None
    }
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

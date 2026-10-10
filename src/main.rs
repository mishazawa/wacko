#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use embassy_nrf as _;
use panic_probe as _;

use defmt::info;
use embassy_executor::Spawner;
use nrf_softdevice::{Softdevice, ble::gatt_server};

use crate::ble::{advertise, init};

mod ble;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Staring..");
    let (sd, server) = init().expect("Can't init BLE");

    spawner.must_spawn(softdevice_task(sd));
    loop {
        match advertise(sd).await {
            Ok(conn) => {
                let e = gatt_server::run(&conn, &server, |_| {}).await;
                info!("gatt_server run exited with error: {:?}", e);
            }
            Err(e) => {
                info!("advertise run exited with error: {:?}", e);
            }
        };
    }
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

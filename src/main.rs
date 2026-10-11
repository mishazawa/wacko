#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use embassy_nrf as _;
use panic_probe as _;

use defmt::info;
use embassy_executor::Spawner;

use crate::ble::Server;

mod ble;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting..");

    let server = Server::new(spawner).expect("Failed to init BLE stack.");

    loop {
        info!("Advertising..");
        match server.advertise().await {
            Ok(conn) => server.on_connection(&conn).await,
            Err(e) => {
                info!("Advertising exited with error: {:?}", e);
            }
        };
    }
}

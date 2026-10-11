#![no_std]
#![no_main]

use cortex_m::Peripherals;
use defmt_rtt as _;
use embassy_futures::select::select;
use embassy_nrf::gpio::{Input, Pull};
// global logger
use embassy_nrf as _;

use panic_probe as _;

use defmt::info;
use embassy_executor::Spawner;

mod ble;
mod btn;

use crate::ble::Server;
use crate::btn::ShutterButton;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting..");
    let server = Server::new(spawner).expect("Failed to init BLE stack.");

    let mut btn = ShutterButton::new(btn_a, &server);

    loop {
        info!("Advertising..");
        match server.advertise().await {
            Ok(conn) => {
                select(server.on_connection(&conn), btn.task(&server, &conn)).await;
            }
            Err(e) => {
                info!("Advertising exited with error: {:?}", e);
            }
        };
    }
}

#![no_std]
#![no_main]

mod ble;
mod btn;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;

use embassy_futures::select;
use microbit_bsp::Microbit;
use panic_probe as _;

use crate::{
    ble::{BleServer, advertise},
    btn::ShutterButton,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Staring..");
    let board = Microbit::default();

    let (sdc, mpsl) = board
        .ble
        .init(board.timer0, board.rng)
        .expect("BLE Stack failed to initialize");

    let (server, mut advertiser) =
        BleServer::start_gatt(spawner, sdc, mpsl).expect("Failed to start GATT server");

    let mut shutter_button = ShutterButton::new(board.btn_a, server);
    loop {
        if let Ok(conn) = advertise(&mut advertiser, &server).await {
            let gatt_task = server.gatt_task(&conn);
            let button_task = shutter_button.task(&conn);
            select::select(gatt_task, button_task).await;
        }
    }
}

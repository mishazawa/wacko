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

use crate::{ble::advertise, btn::ShutterButton};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Staring..");
    let board = Microbit::default();

    let (mut peripheral, server) = ble::init(spawner).await.expect("Can't init BLE stack.");

    let mut shutter_button = ShutterButton::new(board.btn_a, &server);
    loop {
        if let Ok(conn) = advertise(&mut peripheral, &server).await {
            let gatt_task = server.gatt_task(&conn);
            let button_task = shutter_button.task(&conn);
            select::select(gatt_task, button_task).await;
        }
    }
}

#![no_std]
#![no_main]

mod ble;
mod button;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;

use microbit_bsp::Microbit;
use panic_probe as _;

use crate::button::button_handler;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Staring..");
    let board = Microbit::default();

    spawner.must_spawn(button_handler(board.btn_a));

    let (sdc, mpsl) = board
        .ble
        .init(board.timer0, board.rng)
        .expect("BLE Stack failed to initialize");

    spawner.must_spawn(ble::mpsl_task(mpsl));

    ble::run(sdc).await;
}

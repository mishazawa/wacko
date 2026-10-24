#![no_std]
#![no_main]

use defmt::warn;
use defmt_rtt as _;
use embassy_crypto_rustcrypto as _;
use embassy_futures::select::select;
use embassy_nrf::rng::Rng;
use microbit_bsp::*;
use panic_probe as _;

use embassy_executor::Spawner;

mod ble;
mod btn;

use crate::{
    ble::{Server, seed_crypto_rng},
    btn::{ShutterButton, button_task},
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut board = Microbit::default();

    let mut seed = [0u8; 32];
    {
        let mut hw = Rng::new_blocking(board.rng.reborrow());
        hw.set_bias_correction(true);
        hw.blocking_fill_bytes(&mut seed);
    }
    seed_crypto_rng(seed);

    let (sdc, mpsl) = board
        .ble
        .init(board.timer0, board.rng)
        .expect("BLE Stack failed to initialize");

    let (server, mut peripheral) =
        Server::start_gatt(spawner, sdc, mpsl).expect("BLE Stack failed to initialize");

    spawner.must_spawn(button_task(ShutterButton::new(board.btn_a)));

    loop {
        match Server::advertise(&mut peripheral, server).await {
            Ok(conn) => {
                select(Server::gatt_events(&conn), Server::key_task(server, &conn)).await;
            }
            Err(e) => warn!("advertising failed: {:?}", e),
        }
    }
}

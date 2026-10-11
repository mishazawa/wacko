#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use microbit_bsp::*;

use embassy_executor::Spawner;

mod ble;
mod btn;

use crate::btn::ShutterButton;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();

    let mut btn = ShutterButton::new(board.btn_a);

    loop {
        btn.button_state_task().await;
    }
}

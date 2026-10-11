#![no_std]
#![no_main]

use crate::btn::ShutterButton;

use {defmt_rtt as _, panic_probe as _};

use microbit_bsp::*;

use embassy_executor::Spawner;

mod btn;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();

    let mut btn = ShutterButton::new(board.btn_a);

    loop {
        btn.button_state_task().await;
    }
}

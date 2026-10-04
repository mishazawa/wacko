use defmt::info;
use microbit_bsp::embassy_nrf::gpio::Input;

#[embassy_executor::task]
pub async fn button_handler(mut button_a: Input<'static>) {
    loop {
        button_a.wait_for_low().await;
        info!("A AAA!");
        button_a.wait_for_high().await;
    }
}

use defmt::info;
use embassy_time::{Duration, Timer};
use microbit_bsp::Button;

pub struct ShutterButton {
    btn: Button,
}

impl ShutterButton {
    pub fn new(btn: Button) -> Self {
        Self { btn }
    }

    async fn track_btn_state(&mut self) -> ! {
        let debounce = Duration::from_millis(50);
        loop {
            self.btn.wait_for_low().await;
            self.on_btn_press();
            Timer::after(debounce).await;
            self.btn.wait_for_high().await;
            self.on_btn_depress();
            Timer::after(debounce).await;
        }
    }

    pub async fn button_state_task(&mut self) {
        self.track_btn_state().await
    }

    fn on_btn_press(&self) -> () {
        info!("Btn press");
    }

    fn on_btn_depress(&self) -> () {
        info!("Btn depress");
    }
}

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use microbit_bsp::Button;

pub struct ShutterButton {
    btn: Button,
}

#[derive(Clone, Copy, defmt::Format)]
pub enum BtnEvent {
    Pressed,
    Released,
}

pub static BTN_EVENTS: Channel<CriticalSectionRawMutex, BtnEvent, 4> = Channel::new();

impl ShutterButton {
    pub fn new(btn: Button) -> Self {
        Self { btn }
    }

    async fn track_btn_state(&mut self) -> ! {
        let debounce = Duration::from_millis(50);
        loop {
            self.btn.wait_for_low().await;
            let _ = BTN_EVENTS.try_send(BtnEvent::Pressed);
            Timer::after(debounce).await;
            self.btn.wait_for_high().await;
            let _ = BTN_EVENTS.try_send(BtnEvent::Released);
            Timer::after(debounce).await;
        }
    }

    pub async fn button_state_task(&mut self) {
        self.track_btn_state().await
    }
}

#[embassy_executor::task]
pub async fn button_task(mut btn: ShutterButton) {
    btn.button_state_task().await
}

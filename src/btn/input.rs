use defmt::info;
use embassy_nrf::gpio::Input;

use embassy_time::{Duration, Timer};
use nrf_softdevice::ble::Connection;

use crate::ble::Server;

pub type Button = Input<'static>;
pub struct ShutterButton {
    pub name: char,
    pub input: Button,
    pub ble_handle: u16,
}

impl ShutterButton {
    pub fn new(input: Button, server: &Server) -> Self {
        Self {
            name: 'A',
            input,
            ble_handle: server.hid.input_keyboard,
        }
    }

    async fn notify_button_state(&mut self, server: &Server, conn: &Connection) {
        let debounce = Duration::from_millis(50);
        info!("button {} service online", self.name);

        loop {
            self.input.wait_for_low().await;
            info!("button {} pressed", self.name);
            self.send_btn_debounce(0x90, server, conn, &debounce).await;

            self.input.wait_for_high().await;
            info!("button {} depressed", self.name);
            self.send_btn_debounce(0x0, server, conn, &debounce).await;
        }
    }

    async fn send_btn_debounce(
        &self,
        value: u8,
        server: &Server,
        conn: &Connection,
        debounce: &Duration,
    ) {
        server
            .hid
            .notify_button_state(
                self.ble_handle,
                &conn,
                &KeyboardReport::new(value).to_bytes(),
            )
            .expect("Failed to sent button state");
        Timer::after(*debounce).await;
    }

    pub async fn task(&mut self, server: &Server, conn: &Connection) {
        // task for single button
        // never returns
        let _ = self.notify_button_state(server, conn).await;
    }
}

struct KeyboardReport {
    modifiers: u8,
    reserved: u8,
    key_codes: [u8; 6],
}

impl KeyboardReport {
    fn new(val: u8) -> Self {
        Self {
            modifiers: 0,
            reserved: 0,
            key_codes: [val, 0, 0, 0, 0, 0],
        }
    }
    fn to_bytes(&self) -> [u8; 8] {
        [
            self.modifiers,
            self.reserved,
            self.key_codes[0],
            self.key_codes[1],
            self.key_codes[2],
            self.key_codes[3],
            self.key_codes[4],
            self.key_codes[5],
        ]
    }
}

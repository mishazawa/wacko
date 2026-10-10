use defmt::info;
use microbit_bsp::{
    Button,
    embassy_time::{Duration, Timer},
};
use trouble_host::prelude::*;

use crate::ble::BleServer;

pub struct ShutterButton {
    pub name: char,
    pub input: Button,
    pub ble_handle: Characteristic<[u8; 8]>,
}

impl ShutterButton {
    pub fn new(input: Button, server: &BleServer<'static>) -> Self {
        Self {
            name: 'A',
            input,
            ble_handle: server.hid_info.input_keyboard,
        }
    }

    async fn notify_button_state(
        &mut self,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
    ) -> Result<(), BleHostError<Error>> {
        let debounce = Duration::from_millis(50);
        info!("button {} service online", self.name);

        loop {
            self.input.wait_for_low().await;
            info!("button {} pressed", self.name);

            self.ble_handle
                .notify(&conn, &KeyboardReport::new(0x90).to_bytes(), true)
                .await?;
            Timer::after(debounce).await;
            self.input.wait_for_high().await;
            self.ble_handle
                .notify(&conn, &KeyboardReport::new(0x00).to_bytes(), true)
                .await?;
            info!("button {} depressed", self.name);
            Timer::after(debounce).await;
        }
    }

    pub async fn task(&mut self, conn: &GattConnection<'_, '_, DefaultPacketPool>) {
        // task for single button
        // never returns
        let _ = self.notify_button_state(conn).await;
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

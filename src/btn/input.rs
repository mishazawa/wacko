use defmt::info;
use microbit_bsp::{
    Button,
    ble::SoftdeviceError,
    embassy_time::{Duration, Timer},
};
use trouble_host::prelude::*;

use crate::ble::BleServer;

pub struct ShutterButton {
    pub name: char,
    pub input: Button,
    pub ble_handle: Characteristic<bool>,
}

impl ShutterButton {
    pub fn new(input: Button, server: &'static BleServer<'_>) -> Self {
        Self {
            name: 'A',
            input,
            ble_handle: server.hid.button_a,
        }
    }

    async fn notify_button_state(
        &mut self,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
    ) -> Result<(), BleHostError<SoftdeviceError>> {
        let debounce = Duration::from_millis(50);
        info!("button {} service online", self.name);

        loop {
            self.input.wait_for_low().await;
            info!("button {} pressed", self.name);
            self.ble_handle.notify(&conn, &true).await?;
            Timer::after(debounce).await;
            self.input.wait_for_high().await;
            self.ble_handle.notify(&conn, &false).await?;
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

use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use trouble_host::prelude::{service_class::HID, *};

mod gap;
mod gatt;
mod server;

pub use gap::advertise;
pub use server::BleServer;
/*
GAP and GATT

Two profiles that split the work.

GAP, the Generic Access Profile, runs everything up to and including the connection,
so advertising, scanning, roles, and connecting.

GATT, the Generic Attribute Profile, takes over once you’re connected and governs
how data is structured and exchanged.


Rough rule, getting connected is GAP and moving data is GATT.
It’s a rough rule because GAP doesn’t clock out once the connection exists.
It still handles connection parameter updates, security procedures like pairing,
and configuration down at the link layer.

*/

pub const MSP_NORDIC_COMPANY_ID: u16 = 0x0059;
pub const MSP_PAYLOAD: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

const BLE_NAME: &str = "misha_";

/// Size of L2CAP packets (ATT MTU is this - 4)
const L2CAP_MTU: usize = 251;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

pub type BleController = SoftdeviceController<'static>;

pub type BleResources =
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, BleController, DefaultPacketPool>) {
    runner.run().await.expect("Error in BLE task");
}

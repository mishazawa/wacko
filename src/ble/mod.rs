use core::mem;

use defmt::info;
use nrf_softdevice::{Softdevice, ble::gatt_server::RegisterError, raw};

mod constants;
mod gap;
mod gatt;
mod server;

pub use gap::advertise;
pub use server::Server;

pub const SERIAL_NUMBER: &'static str = "14881488";

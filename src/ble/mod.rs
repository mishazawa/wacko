mod constants;
mod crypto;
mod gatt;
mod server;
// must be less than 24 =)
// safer than compiler -> commentary from developer
pub const MANUFACTURER_NAME_STRING: &str = "Sorokin";
pub const MODEL_NUMBER_DATA: &str = "Norma";
pub const SERIAL_NUMBER: &str = "1979-1983";
pub const BT_NAME: &str = "SorokinKbd";

pub use crypto::seed_crypto_rng;
pub use server::Server;

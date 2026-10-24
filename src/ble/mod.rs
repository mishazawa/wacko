mod constants;
mod crypto;
mod gatt;
mod server;
// must be less than 24 =)
// safer than compiler -> commentary from developer
pub const MANUFACTURER_NAME_STRING: &str = "suka";
pub const MODEL_NUMBER_DATA: &str = "idi_na_hui";
pub const SERIAL_NUMBER: &str = "1488";
pub const BT_NAME: &str = "NAHUI";

pub use crypto::seed_crypto_rng;
pub use server::Server;

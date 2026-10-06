use trouble_host::prelude::*;

#[gatt_service(uuid = "260279e7-a5dd-447b-9bd8-e624ef464d6e")]
pub struct ButtonService {
    #[characteristic(uuid = "c665eb11-eee4-452b-9047-a98a3916bd80", read, notify)]
    pub button_a: bool,
}

use defmt::info;
use trouble_host::prelude::*;

use crate::ble::{BLE_NAME, BleServer};

pub async fn advertise<'a, 'b, C: Controller>(
    peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
    server: &'b BleServer<'b>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];

    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;

    info!("[adv] advertising");

    // this is very important line!! connection -> gatt connection
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

use defmt::info;
use trouble_host::prelude::*;

use crate::ble::{BLE_NAME, BleServer, MSP_NORDIC_COMPANY_ID, MSP_PAYLOAD};

pub async fn advertise<'a, 'b, C: Controller>(
    peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
    server: &'b BleServer<'b>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<C::Error>> {
    // Legacy Indirect Advertisement
    const GAP_ADV_LIMIT: usize = 31;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];

    let ad_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x12, 0x18]]),
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;

    let mut scan_resp = [0; GAP_ADV_LIMIT];

    let sr_len = AdStructure::encode_slice(
        &[AdStructure::ManufacturerSpecificData {
            company_identifier: MSP_NORDIC_COMPANY_ID,
            payload: &MSP_PAYLOAD,
        }],
        &mut scan_resp[..],
    )?;

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[0..ad_len],
                scan_data: &scan_resp[0..sr_len],
            },
        )
        .await?;

    info!("[adv] advertising");

    // this is very important line!! connection -> gatt connection
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

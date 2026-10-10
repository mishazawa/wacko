use defmt::info;
use trouble_host::prelude::*;

use crate::ble::{BLE_NAME, BleResources, BleServer, MSP_NORDIC_COMPANY_ID, MSP_PAYLOAD};

pub async fn advertise<'a, C: Controller>(
    controller: C,
) -> Result<
    (
        Runner<'a, C, DefaultPacketPool>,
        Advertiser<'a, C, DefaultPacketPool>,
    ),
    BleHostError<C::Error>,
>
where
    C: Controller + 'a,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let mut resources: BleResources = HostResources::new();

    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .build();

    let runner = stack.runner();
    let mut peripheral = stack.peripheral();

    // Legacy Indirect Advertisement
    const GAP_ADV_LIMIT: usize = 31;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];

    let ad_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED), // flags
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),                // name
            AdStructure::Unknown {
                ty: 0x19,
                data: &[0xc1, 0x03],
            }, // kbd
            AdStructure::CompleteServiceUuids16(&[[0x12, 0x18]]),               // hid
        ],
        &mut advertiser_data[..],
    )?;

    let mut scan_resp = [0; GAP_ADV_LIMIT];

    let sr_len = AdStructure::encode_slice(
        &[
            AdStructure::ManufacturerSpecificData {
                company_identifier: MSP_NORDIC_COMPANY_ID,
                payload: &MSP_PAYLOAD,
            },
            AdStructure::CompleteServiceUuids16(&[
                service::DEVICE_INFORMATION.to_le_bytes(),
                service::BATTERY.to_le_bytes(),
                service::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                service::GATT.to_le_bytes(),
            ]),
        ],
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

    Ok((runner, advertiser))
    // info!("[adv] advertising");

    // // // this is very important line!! connection -> gatt connection
    // let conn = advertiser.accept().await?.with_attribute_server(server)?;
    // info!("[adv] connection established");
    // Ok(conn)
}

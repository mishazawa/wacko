use trouble_host::prelude::{service_class::HID, *};

/*
To draw parallels to REST APIs, GATT profiles would be web endpoints,
while the ATT protocol would be the GET, POST, and PUT request types.

The GATT Profile is used by all BLE devices that want to transport data.
However, the GATT Profile also serves as a mechanism of interoperability,
allowing clients and servers to exchange information without being
specifically designed for each other.

A Profile is simply a group of one or more services needed to fulfill a use case.
Profiles are entirely conceptual. Devices do not declare that they conform to a profile,
and profiles do not have concrete entries in a peripheral’s GATT Table.

---

Services are the top level of concrete elements in a GATT table. Services are
simply a collection of characteristics. The Bluetooth SIG defines standard
services as having mandatory characteristics or optional characteristics.
Custom services can include whatever characteristics they want, as they are
not attempting to conform to an interoperability standard.

Services are identified by their UUID, which can be 16, 32, or 128 bits.
16-bit and 32-bit UUIDs are assigned by the Bluetooth SIG either as
a common service for interoperability or as a proprietary service whose UUID
was purchased by a 3rd party. 128-bit UUIDs are unassigned and can be used by anyone.
The only exception for 128-bit UUIDs is ‘XXXXXXXX-0000-1000-8000-00805F9B34FB’,
which is the base UUID used for all 16-bit and 32-bit UUIDs. 32-bit UUIDs
use bytes 0-4, while 16-bit UUIDs use bytes 2-4 with bytes 0-2 set to zero.



*/

// SDP
#[gatt_service(uuid = HID)]
pub struct ButtonConfigurationService {
    //todo
}

#[gatt_service(uuid = "1488")]
pub struct ButtonService {
    #[characteristic(uuid = "1489", read, notify)]
    pub button_a: bool,
}

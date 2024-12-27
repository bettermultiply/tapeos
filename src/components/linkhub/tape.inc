// Service UUID for tapeos
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0xFEEDC0DE);
// Characteristic UUID for tapeos
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0xF00DC0DE00001);

// Manufaturer id for LE advertise.
#[allow(dead_code)]
const TAPE_MANUFACTURER_ID: u16 = 0xf00d;

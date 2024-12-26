// Service UUID for tapeos
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001523_1212_efde_1523_785feabcd123);
// Characteristic UUID for tapeos
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001524_1212_efde_1523_785feabcd123);

// Manufaturer id for LE advertise.
#[allow(dead_code)]
const TAPE_MANUFACTURER_ID: u16 = 0x0000;

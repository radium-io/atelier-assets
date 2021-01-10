use uuid::Uuid;

extern crate atelier_core;
extern crate bincode;
extern crate serde_json;

use atelier_core::AssetUuid;

const BYTES: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

#[test]
fn serialize_asset_uuid_string() {
    let uuid = AssetUuid(Uuid::from_bytes(BYTES));
    let result = serde_json::to_string(&uuid).unwrap();
    assert_eq!(
        "\"01020304-0506-0708-090a-0b0c0d0e0f10\"".to_string(),
        result
    );
}

#[test]
fn serialize_asset_uuid_binary() {
    let uuid = AssetUuid(Uuid::from_bytes(BYTES));
    let result: Vec<u8> = bincode::serialize(&uuid).unwrap();
    assert_eq!(BYTES.to_vec(), result);
}

#[test]
fn deserialize_asset_uuid_string() {
    let string = "\"01020304-0506-0708-090a-0b0c0d0e0f10\"";
    let result: AssetUuid = serde_json::from_str(string).unwrap();
    let expected = AssetUuid(Uuid::from_bytes(BYTES));
    assert_eq!(expected, result);
}

#[test]
fn deserialize_asset_uuid_binary() {
    assert_eq!(
        AssetUuid(Uuid::from_bytes(BYTES)),
        bincode::deserialize(&BYTES).unwrap()
    );
}

const ASSET_TYPE_BYTES: [u8; 16] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3];

#[test]
fn serialize_type_uuid_string() {
    let uuid = atelier_core::AssetTypeId(Uuid::from_bytes(ASSET_TYPE_BYTES));
    let result = serde_json::to_string(&uuid).unwrap();
    assert_eq!(
        "\"03010401-0509-0206-0503-050809070903\"".to_string(),
        result
    );
}

#[test]
fn serialize_type_uuid_binary() {
    let uuid = atelier_core::AssetTypeId(Uuid::from_bytes(ASSET_TYPE_BYTES));
    let result: Vec<u8> = bincode::serialize(&uuid).unwrap();
    assert_eq!(ASSET_TYPE_BYTES.to_vec(), result);
}

#[test]
fn deserialize_type_uuid_string() {
    let string = "\"03010401-0509-0206-0503-050809070903\"";
    let result: atelier_core::AssetTypeId = serde_json::from_str(string).unwrap();
    let expected = atelier_core::AssetTypeId(Uuid::from_bytes(ASSET_TYPE_BYTES));
    assert_eq!(expected, result);
}

#[test]
fn deserialize_type_uuid_binary() {
    let data = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3];
    let result: atelier_core::AssetTypeId = bincode::deserialize(&data).unwrap();
    assert_eq!(
        atelier_core::AssetTypeId(Uuid::from_bytes(ASSET_TYPE_BYTES)),
        result
    );
}

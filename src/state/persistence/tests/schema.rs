use super::*;

#[test]
fn a_save_version_beyond_the_supported_integer_range_is_rejected() {
    let value = serde_json::to_value(SaveGame {
        version: SAVE_VERSION,
        campaign: campaign(),
    })
    .unwrap();
    let mut value = value;
    value["version"] = serde_json::Value::from(u64::from(u32::MAX) + 1);

    let error = load_from_json(&serde_json::to_string(&value).unwrap()).unwrap_err();

    assert!(error
        .to_string()
        .contains("unsupported save version 4294967296"));
}

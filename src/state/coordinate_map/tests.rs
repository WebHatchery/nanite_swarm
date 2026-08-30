use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Holder {
    #[serde(with = "crate::state::coordinate_map")]
    map: HashMap<(i32, i32), f32>,
}

#[test]
fn coordinate_maps_use_string_keys_and_round_trip() {
    let holder = Holder {
        map: HashMap::from([((-2, 7), 3.5), ((4, -1), 8.0)]),
    };
    let json = serde_json::to_string(&holder).unwrap();

    assert!(json.contains("\"-2,7\""));
    assert_eq!(serde_json::from_str::<Holder>(&json).unwrap(), holder);
}

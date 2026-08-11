// Guards the open-enum rewrite in scripts/fix-open-enums.py: payers return values outside
// the spec's closed enum lists (e.g. the singular "Visit" where UnitForMeasurement only
// declares "Visits"), and without a fallback variant one such string fails deserialization
// of the entire response. If a regeneration ever drops the `#[serde(untagged)]
// UnknownValue(String)` variant, these tests fail.
#![cfg(feature = "healthcare")]

use autogen_stedi::healthcare::models::UnitForMeasurement;

#[test]
fn spec_values_still_parse_to_named_variants() {
    let parsed: UnitForMeasurement = serde_json::from_str("\"Days\"").unwrap();
    assert_eq!(parsed, UnitForMeasurement::Days);
}

#[test]
fn non_compliant_value_falls_back_to_unknown() {
    let parsed: UnitForMeasurement = serde_json::from_str("\"Visit\"").unwrap();
    assert_eq!(parsed, UnitForMeasurement::UnknownValue("Visit".to_string()));
}

#[test]
fn unknown_round_trips_verbatim() {
    let parsed: UnitForMeasurement = serde_json::from_str("\"Visit\"").unwrap();
    assert_eq!(serde_json::to_string(&parsed).unwrap(), "\"Visit\"");
    assert_eq!(parsed.to_string(), "Visit");
}

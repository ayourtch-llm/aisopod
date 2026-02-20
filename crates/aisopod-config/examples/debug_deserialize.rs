use aisopod_config::AisopodConfig;
use json5;
use serde_json;

fn main() {
    let content = std::fs::read_to_string(
        "/home/ayourtch/rust/aisopod/crates/aisopod-config/tests/fixtures/valid_full.json5",
    )
    .unwrap();

    // Parse as JSON5
    let value: serde_json::Value = json5::from_str(&content).expect("Failed to parse JSON5");

    println!("Parsed JSON5 successfully");

    // Try to deserialize to AisopodConfig
    let result: Result<AisopodConfig, _> = serde_json::from_value(value);

    match result {
        Ok(config) => println!("Deserialization succeeded!"),
        Err(e) => println!("Deserialization failed: {}", e),
    }
}

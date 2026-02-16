use json5;
use serde_json;

fn main() {
    let content = std::fs::read_to_string("/home/ayourtch/rust/aisopod/crates/aisopod-config/tests/fixtures/valid_full.json5").unwrap();
    
    // Parse as JSON5
    let value: serde_json::Value = json5::from_str(&content).expect("Failed to parse JSON5");
    
    // Print the channels field if it exists
    if let serde_json::Value::Object(map) = &value {
        if let Some(channels) = map.get("channels") {
            println!("channels field: {:?}", channels);
        } else {
            println!("channels field is MISSING");
        }
        println!("All keys: {:?}", map.keys().collect::<Vec<_>>());
    }
}

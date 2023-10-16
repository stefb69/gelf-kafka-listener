use std::collections::HashMap;
use flate2::read::GzDecoder;
use std::io::Read;

pub type GelfMessage = HashMap<String, serde_json::Value>;

pub fn decode_gelf_message(compressed_data: &[u8]) -> Result<GelfMessage, Box<dyn std::error::Error>> {
    let mut d = GzDecoder::new(compressed_data);
    let mut decompressed_data = String::new();
    d.read_to_string(&mut decompressed_data)?;

    let message: GelfMessage = serde_json::from_str(&decompressed_data)?;
    Ok(message)
}

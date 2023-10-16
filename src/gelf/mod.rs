use std::collections::HashMap;
use flate2::read::GzDecoder;
use std::io::Read;

pub type GelfMessage = HashMap<String, serde_json::Value>;

fn is_gzipped(data: &[u8]) -> bool {
    data.starts_with(&[0x1f, 0x8b])
}

pub fn decode_gelf_message(data: &[u8]) -> Result<GelfMessage, Box<dyn std::error::Error>> {
    let mut _data = String::new();
    if is_gzipped(&data) {
        let mut d = GzDecoder::new(data);
        d.read_to_string(&mut _data)?;    
    } else {
        _data = String::from_utf8_lossy(&data).to_string();
    }
    let message: GelfMessage = serde_json::from_str(&_data)?;
    Ok(message)
}

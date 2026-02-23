use moth_filter::{MothDataJson, SpeciesData};

use crate::zstd::decode_zstd;

pub fn init() -> Result<Vec<SpeciesData>, Box<dyn std::error::Error>> {
    let moth_data_file_decompressed =
        decode_zstd(include_bytes!("../../assets/moth_data.json.zst")).unwrap();
    let moth_data_file_str = str::from_utf8(&moth_data_file_decompressed).unwrap();

    let moth_data: MothDataJson = serde_json::from_str(moth_data_file_str).unwrap();
    return Ok(moth_data);
}

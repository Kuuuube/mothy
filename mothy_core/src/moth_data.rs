use crate::{structs::MothData, zstd::decode_zstd_json};

pub fn moth_data_init() -> Result<MothData, Box<dyn std::error::Error>> {
    return Ok(MothData {
        moth_data: decode_zstd_json(include_bytes!("../../assets/moth_data.json.zst"))?,
        butterfly_blacklist: decode_zstd_json(include_bytes!(
            "../../assets/butterfly_blacklist.json.zst"
        ))?,
    });
}

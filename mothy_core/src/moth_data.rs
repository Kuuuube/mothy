use moth_filter::{ButterflyBlacklist, MothDataJson, SpeciesData};

use crate::zstd::decode_zstd;

pub fn moth_data_init() -> Result<Vec<SpeciesData>, Box<dyn std::error::Error>> {
    let moth_data_file_decompressed =
        decode_zstd(include_bytes!("../../assets/moth_data.json.zst")).unwrap();
    let moth_data_file_str = str::from_utf8(&moth_data_file_decompressed).unwrap();

    let moth_data: MothDataJson = serde_json::from_str(moth_data_file_str).unwrap();
    return Ok(moth_data);
}

pub fn butterfly_blacklist_init() -> Result<ButterflyBlacklist, Box<dyn std::error::Error>> {
    let butterfly_blacklist_file_decompressed =
        decode_zstd(include_bytes!("../../assets/butterfly_blacklist.json.zst")).unwrap();
    let butterfly_blacklist_file_str =
        str::from_utf8(&butterfly_blacklist_file_decompressed).unwrap();

    let butterfly_blacklist_data: ButterflyBlacklist =
        serde_json::from_str(butterfly_blacklist_file_str).unwrap();
    return Ok(butterfly_blacklist_data);
}

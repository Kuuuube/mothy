use crate::{structs::ScoresData, zstd::decode_zstd};

pub fn init() -> Result<Vec<ScoresData>, Box<dyn std::error::Error>> {
    let scores_file_decompressed =
        decode_zstd(include_bytes!("../../assets/scores_data.json.zst")).unwrap();
    let scores_file_str = str::from_utf8(&scores_file_decompressed).unwrap();

    let scores_data: Vec<ScoresData> = serde_json::from_str(scores_file_str).unwrap();
    return Ok(scores_data);
}

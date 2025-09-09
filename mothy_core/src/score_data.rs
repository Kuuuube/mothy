use crate::structs::ScoresData;

pub fn init() -> Result<Vec<ScoresData>, Box<dyn std::error::Error>> {
    let scores_file_decompressed = decode_zstd(include_bytes!("../../assets/scores_data.json.zst")).unwrap();
    let scores_file_str = str::from_utf8(&scores_file_decompressed).unwrap();
    // dbg!(scores_file_str);

    let scores_data: Vec<ScoresData> = serde_json::from_str(scores_file_str).unwrap();
    return Ok(scores_data);
}

fn decode_zstd(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bound = zstd_safe::decompress_bound(&data).expect("zstd_safe::decompress_bound failed");
    let mut decompressed: Vec<u8> = Vec::with_capacity(bound.try_into()?);
    zstd_safe::decompress(&mut decompressed, &data).expect("zstd_safe::decompress failed");
    return Ok(decompressed);
}

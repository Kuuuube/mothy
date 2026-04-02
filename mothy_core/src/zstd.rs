pub fn decode_zstd(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bound = zstd_safe::decompress_bound(data).expect("zstd_safe::decompress_bound failed");
    let mut decompressed: Vec<u8> = Vec::with_capacity(bound.try_into()?);
    zstd_safe::decompress(&mut decompressed, data).expect("zstd_safe::decompress failed");
    Ok(decompressed)
}

pub fn decode_zstd_json<T>(data: &[u8]) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let decoded_bytes = decode_zstd(data)?;
    let decoded_str = str::from_utf8(&decoded_bytes)?;
    let decoded_json: T = serde_json::from_str(decoded_str)?;
    Ok(decoded_json)
}

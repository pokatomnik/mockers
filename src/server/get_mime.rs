pub fn get_mime(source: &Vec<u8>) -> String {
    return infer::get(source)
        .map(|t| t.mime_type().to_string())
        .unwrap_or("application/octet-stream".to_string());
}

use mimetype_detector::detect;

pub fn get_mime(source: &Vec<u8>) -> String {
    return detect(source).mime().to_string();
}

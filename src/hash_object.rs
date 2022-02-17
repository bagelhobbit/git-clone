use super::object_util;
use sha1::Sha1;
use std::fs;
use std::str;

/// Returns the SHA1 hash of the passed file.
pub fn hash_object(filepath: &str) -> std::string::String {
    let file = fs::read(filepath).expect("Unable to read file");
    // Assume a valid UTF-8 file
    let content = str::from_utf8(&file).unwrap();

    // Assume type is blob if not specified
    let object_type = "blob";
    let header = format!("{} {}", object_type, content.len());
    let store = format!("{}\0{}", header, content);

    Sha1::from(store.as_bytes()).hexdigest()
}

/// Like `hash_object` returns the SHA1 hash of the passed file,
/// but also writes the object to the object database.
pub fn write_hash_object(filepath: &str) -> std::string::String {
    let file = fs::read(filepath).expect("Unable to read file");
    // Assume a valid UTF-8 file
    let content = str::from_utf8(&file).unwrap();

    // Assume type is blob if not specified
    let object_type = "blob";
    let header = format!("{} {}", object_type, content.len());
    let store = format!("{}\0{}", header, content);

    let hash = Sha1::from(store.as_bytes()).hexdigest();

    object_util::write_object_file(&hash, store.as_bytes());

    hash
}

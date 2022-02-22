use super::object_util;
use sha1::Sha1;
use std::fs;
use std::str;

/// Returns the SHA1 hash of the passed file.
pub fn generate_hash(filepath: &str) -> std::string::String {
    let file = fs::read(filepath).expect("Unable to read file");
    // Assume a valid UTF-8 file
    let content = str::from_utf8(&file).unwrap();

    let store = create_store(content);
    Sha1::from(store.as_bytes()).hexdigest()
}

/// Like `hash_object` returns the SHA1 hash of the passed file,
/// but also writes the object to the object database.
pub fn write_hash_object(filepath: &str) -> std::string::String {
    let file = fs::read(filepath).expect("Unable to read file");
    // Assume a valid UTF-8 file
    let content = str::from_utf8(&file).unwrap();

    let store = create_store(content);
    let hash = Sha1::from(store.as_bytes()).hexdigest();

    object_util::write_object_file(&hash, store.as_bytes());

    hash
}

/// Creates the store that can be used to generate an object hash
fn create_store(content: &str) -> String {
    // Assume type is blob if not specified
    let object_type = "blob";
    let header = format!("{} {}", object_type, content.len());
    format!("{}\0{}", header, content)
}

#[cfg(test)]
mod tests {
    use crate::hash_object::create_store;

    #[test]
    fn test_create_store_blob() {
        let content = "Hello, World";
        let store = "blob 12\0Hello, World";
        assert_eq!(store, create_store(content));
    }
}

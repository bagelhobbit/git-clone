use flate2::write::ZlibEncoder;
use sha1::Sha1;
use std::fs;
use std::io::Write;
use std::str;

/// Returns the SHA1 hash of the passed file.
///
/// # Arguments
///
/// * `filepath` - file to hash
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
///
/// # Arguments
///
/// * `filepath` - file to hash
pub fn write_hash_object(filepath: &str) -> std::string::String {
    let file = fs::read(filepath).expect("Unable to read file");
    // Assume a valid UTF-8 file
    let content = str::from_utf8(&file).unwrap();

    // Assume type is blob if not specified
    let object_type = "blob";
    let header = format!("{} {}", object_type, content.len());
    let store = format!("{}\0{}", header, content);

    let hash = Sha1::from(store.as_bytes()).hexdigest();

    // Use first 2 digits as the direcectory, and the rest as the file name
    let out_dir = &hash[..2];
    let out_filename = &hash[3..];
    let out_dir_path = format!("gitrs/objects/{}", &out_dir);
    let out_path = format!("{}/{}", &out_dir_path, &out_filename);

    fs::create_dir(&out_dir_path).expect("Could not create new directory for object database");
    let out_file = fs::File::create(&out_path).expect("Could not create object file");

    let mut encoder = ZlibEncoder::new(out_file, flate2::Compression::default());
    encoder
        .write_all(store.as_bytes())
        .expect("Could not write object file");

    return hash;
}

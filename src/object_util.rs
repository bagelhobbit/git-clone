use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::fmt;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::str;
use std::string::String;
use std::vec::Vec;

/// The possible types for a git object
///
/// * Blob
/// * Tree
/// * Commit
#[derive(Debug, PartialEq)]
pub enum Object {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Object::Blob => write!(f, "blob"),
            Object::Tree => write!(f, "tree"),
            Object::Commit => write!(f, "commit"),
        }
    }
}

/// Given a git header, returns the type of the object
///
/// # Format
///
/// An object header is the type of object, a space, the size of the contents in bytes, then a null byte
pub fn get_header_type(header: &[u8]) -> Object {
    // space (' ') is 32(dec) in ascii
    let mut split = header.split(|c| c == &32u8);
    let header_type = split.next().expect("Invalid header");

    let tmp = str::from_utf8(&header_type).unwrap();

    match tmp {
        _ if tmp == "blob" => return Object::Blob,
        _ if tmp == "tree" => return Object::Tree,
        _ if tmp == "commit" => return Object::Commit,
        _ => panic!("Invalid header"),
    };
}

/// Given a git header, returns the size of an object's contents
///
/// # Format
///
/// An object header is the type of object, a space, the size of the contents in bytes, then a null byte
pub fn get_header_size(header: &[u8]) -> &str {
    // space (' ') is 32(dec) in ascii
    let mut split = header.split(|c| c == &32u8);
    let _ = split.next().expect("Invalid header");
    let header_size = split.next().expect("Invalid header");

    str::from_utf8(&header_size).unwrap()
}

/// Given an object hash, return its relative path
pub fn get_object_path(object_hash: &str) -> String {
    let object_dir = "gitrs/objects/";
    // The first 2 characters of the hash is the directory the object is stored in
    let hash_dir = &object_hash[..2];
    // The remaing characters are the filename
    let filename = &object_hash[2..];
    format!("{}{}/{}", &object_dir, &hash_dir, &filename)
}

/// Given an object hash, return the files contents
pub fn read_object_file(object_hash: &str) -> Vec<u8> {
    let path = get_object_path(object_hash);
    fs::read(&path).expect(&format!("Could not read object file: {}", path))
}

/// Write the given store out to the object database, using the object has as its key
///
/// A store consists of a header and the content to be stored
/// A header is the object type and length of the content
pub fn write_object_file(object_hash: &str, store: &[u8]) {
    // Use first 2 digits as the direcectory, and the rest as the file name
    let out_dir = &object_hash[..2];
    let out_filename = &object_hash[2..];
    let out_dir_path = format!("gitrs/objects/{}", &out_dir);
    let out_path = format!("{}/{}", &out_dir_path, &out_filename);

    let dir_exists = Path::new(&out_dir_path).exists();
    let path_exists = Path::new(&out_path).exists();

    if !dir_exists {
        fs::create_dir(&out_dir_path).expect("Could not create new directory for object database");
    }

    if !path_exists {
        let out_file = fs::File::create(&out_path).expect("Could not create object file");

        let mut encoder = ZlibEncoder::new(out_file, flate2::Compression::default());
        encoder
            .write_all(store)
            .expect("Could not write object file");
    }
}

/// Decode/decompress a Zlib compressed byte sequence
pub fn decode_object(object: Vec<u8>) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(&object[..]);
    let mut decompressed: Vec<u8> = Vec::new();

    // Assume file is valid
    decoder.read_to_end(&mut decompressed).unwrap();

    decompressed
}

/// Converts a u8 byte array to a string of hex bytes
///
/// Single digits are zero padded
pub fn to_hex_string(bytes: &[u8]) -> String {
    let mut hex_string = String::new();
    for item in bytes {
        // Zero pad any single digit hex values
        hex_string += &format!("{:0>2x}", item);
    }
    hex_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_header_type_with_blob() {
        let header = [0x62, 0x6C, 0x6F, 0x62, 32u8, 0x31, 0x35, 0x0];
        assert_eq!(Object::Blob, get_header_type(&header));
    }

    #[test]
    fn test_get_header_type_with_tree() {
        let header = [0x74, 0x72, 0x65, 0x65, 32u8, 0x31, 0x35, 0x0];
        assert_eq!(Object::Tree, get_header_type(&header));
    }

    #[test]
    fn test_get_header_type_with_commit() {
        let header = [0x63, 0x6F, 0x6D, 0x6D, 0x69, 0x74, 32u8, 0x31, 0x35, 0x0];
        assert_eq!(Object::Commit, get_header_type(&header));
    }

    #[test]
    #[should_panic(expected = "Invalid header")]
    fn test_get_header_type_with_invalid_object() {
        let header = [0x65, 32u8, 0x31, 0x35, 0x0];
        let _ = get_header_type(&header);
    }

    #[test]
    #[should_panic(expected = "Invalid header")]
    fn test_get_header_type_with_invalid_header() {
        let header = [0x62, 0x6C, 0x6F, 0x62, 0x0];
        let _ = get_header_type(&header);
    }

    #[test]
    fn test_get_header_size_when_valid() {
        let header = [0x62, 0x6C, 0x6F, 0x62, 32u8, 0x31, 0x35, 0x0];
        assert_eq!("15\0", get_header_size(&header))
    }

    #[test]
    #[should_panic(expected = "Invalid header")]
    fn test_get_header_size_with_short_header() {
        let header = [0x62, 0x6C, 0x6F, 0x62];
        let _ = get_header_size(&header);
    }

    #[test]
    fn test_get_header_size_with_extra_spaces() {
        let header = [0x62, 0x6C, 0x6F, 0x62, 32u8, 32u8, 0x31, 0x35, 0x0];
        assert_eq!("", get_header_size(&header));
    }

    #[test]
    fn test_to_hex_string() {
        let array = [1u8, 2u8, 3u8, 45u8];
        assert_eq!("0102032d", to_hex_string(&array));
    }
}

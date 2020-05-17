use flate2::read::ZlibDecoder;
use std::fmt;
use std::fs;
use std::str;
use std::vec::Vec;
use std::io::Read;

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
pub fn get_header_size(header: &[u8]) -> &str {
    // space (' ') is 32(dec) in ascii
    let mut split = header.split(|c| c == &32u8);
    let _ = split.next().expect("Invalid header");
    let header_size = split.next().expect("Invalid header");

    str::from_utf8(&header_size).unwrap()
}

/// Given an object hash, return the files contents
pub fn read_object_file(object_hash: &str) -> Vec<u8> {
    let object_dir = "gitrs/objects/";
    // let object_dir = ".git/objects/";
    // The first 2 characters of the hash is the directory the object is stored in
    let hash_dir = &object_hash[..2];
    // The remaing characters are the filename
    let filename = &object_hash[2..];
    let path = format!("{}{}/{}", &object_dir, &hash_dir, &filename);

    fs::read(&path).expect("Could not read object file")
}

/// Decode/decompress a Zlib compressed byte sequence
pub fn decode_object(object: Vec<u8>) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(&object[..]);
    let mut decompressed: Vec<u8> = Vec::new();

    // Assume file is valid
    decoder.read_to_end(&mut decompressed).unwrap();

    decompressed
}

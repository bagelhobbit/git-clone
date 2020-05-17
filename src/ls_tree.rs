use super::object_util;
use std::str;
use std::string::String;

/// Returns the contents of a `Tree` object
pub fn ls_tree(object_hash: &str) -> String {
    let git_object = object_util::read_object_file(object_hash);

    let decoded = object_util::decode_object(git_object);

    format_tree(&decoded)
}

/// Convience method to display a tree.
/// 
/// If you have the object's hash, use `ls_tree` instead.
pub fn format_tree(decoded: &Vec<u8>) -> String {
    let mut formatted_tree = String::new();

    // A tree is a zlib compressed file of a header and a list of file information
    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    // File information is the permissions, space, the filename, null byte, the hex hash (20 bytes)
    let mut split = decoded.split(|num| num == &0u8);

    let header = split.next().unwrap();
    let object_type = object_util::get_header_type(&header);
    if object_type != object_util::Object::Tree {
        return "fatal: not a tree object".to_owned();
    }

    let mut permissions = Vec::<&str>::new();
    let mut filenames = Vec::<&str>::new();
    let mut hashes = Vec::<String>::new();

    // Handle first section since it doesn't start with a hash
    let first_section = split.next().unwrap();
    // Look for the space that separates the permissions from the filename
    let first_index = first_section.iter().position(|&num| num == 32u8).unwrap();
    permissions.push(str::from_utf8(&first_section[..first_index]).unwrap());
    // Add one to skip over space
    filenames.push(str::from_utf8(&first_section[(first_index + 1)..]).unwrap());

    for section in split {
        if section.len() > 0 {
            hashes.push(to_hex_string(&section[..20]));
            if section.len() > 20 {
                // Look for the space that separates the permissions from the filename
                // We don't want to index into the hash so use a a slice to go past the hash bytes
                let slice_index = section[20..].iter().position(|&num| num == 32u8).unwrap();
                // Add 20 to move the index past the hash section
                let index = slice_index + 20;
                permissions.push(str::from_utf8(&section[20..index]).unwrap());
                // Add one to skip over space
                filenames.push(str::from_utf8(&section[(index + 1)..]).unwrap());
            }
        }
    }

    for i in 0..(permissions.len() - 1) {
        let git_object = object_util::read_object_file(&hashes[i]);
        let decoded = object_util::decode_object(git_object);
        let header = decoded.split(|num| num == &0u8).next().unwrap();
        let header_type = object_util::get_header_type(&header);
        formatted_tree += &format!("{:0>6} {} {}\t{}\n", permissions[i], header_type, hashes[i], filenames[i]);
    }

    formatted_tree
}

fn to_hex_string(bytes: &[u8]) -> String {
    let mut hex_string = String::new();
    for item in bytes {
        // Zero pad any single digit hex values
        hex_string += &format!("{:0>2x}", item);
    }
    hex_string
}

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
pub fn format_tree(decoded: &[u8]) -> String {
    let mut formatted_tree = String::new();

    // A tree is a zlib compressed file of a header and a list of file information
    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    // File information is the permissions, space, the filename, null byte, the hex hash (20 bytes)
    let mut split = decoded.split(|num| num == &0u8);

    let header = split.next().unwrap();
    let object_type = object_util::get_header_type(header);
    if object_type != object_util::Object::Tree {
        return "fatal: not a tree object".to_owned();
    }

    let mut permissions = Vec::<&str>::new();
    let mut filenames = Vec::<&str>::new();
    let mut hashes = Vec::<String>::new();

    // Handle first section since it doesn't start with a hash
    let mut first_section = split.next().unwrap();

    // Make sure first index can find something and we don't index into an empty array
    // There's probably a better way to fix this by not 'unwrap'-ing everything, but I don't want to figure it out
    first_section = if first_section.is_empty() {
        &[32u8]
    } else {
        first_section
    };

    // Look for the space that separates the permissions from the filename
    let first_index = first_section.iter().position(|&num| num == 32u8).unwrap();
    permissions.push(str::from_utf8(&first_section[..first_index]).unwrap());
    // Add one to skip over space
    filenames.push(str::from_utf8(&first_section[(first_index + 1)..]).unwrap());

    for section in split {
        if !section.is_empty() {
            hashes.push(object_util::to_hex_string(&section[..20]));
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
        let header_type = object_util::get_header_type(header);
        formatted_tree += &format!(
            "{:0>6} {} {}\t{}\n",
            permissions[i], header_type, hashes[i], filenames[i]
        );
    }

    formatted_tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tree_handles_empty_tree() {
        let decoded = [116u8, 114, 101, 101, 32, 48, 0];
        assert_eq!("", format_tree(&decoded));
    }
}

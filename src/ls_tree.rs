use super::object_util;
use std::str;
use std::string::String;

/// Returns the contents of a [`Tree`] object
pub fn ls_tree(object_hash: &str) -> String {
    let git_object = object_util::read_object_file(object_hash);

    let decoded = object_util::decode_object(git_object);

    format_tree(&decoded)
}

/// Convience method to display a tree.
///
/// If you have the object's hash, use [`ls_tree`] instead.
pub fn format_tree(decoded: &[u8]) -> String {
    let mut formatted_tree = String::new();

    // A tree is a zlib compressed file of a header and a list of file information
    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    // File information is the permissions, space, the filename, null byte, the hex hash (20 bytes)
    let found_header = decoded.iter().enumerate().find(|x| x.1 == &0u8);
    let header_end: usize;
    match found_header {
        Some((i, _)) => header_end = i,
        None => return "fatal: invalid header".to_string(),
    }

    let header = &decoded[..header_end];

    let object_type = object_util::get_header_type(header);
    if object_type != object_util::Object::Tree {
        return "fatal: not a tree object".to_string();
    }

    let mut permissions = Vec::<&str>::new();
    let mut filenames = Vec::<&str>::new();
    let mut hashes = Vec::<String>::new();

    // Add 1 to skip over the null byte we searched for earlier
    let mut next_parse = &decoded[header_end + 1..];
    while !next_parse.is_empty() {
        let found_section = next_parse.iter().enumerate().find(|x| x.1 == &0u8);
        let mut section_end: usize;
        match found_section {
            Some((i, _)) => section_end = i,
            None => section_end = 0,
        }

        if section_end == 0 {
            break;
        }

        let section = &next_parse[..section_end];
        // Increase section_end past the null byte
        // This slightly simplifies the math to read the hash
        section_end += 1;

        // Permissions should be 6 bytes
        permissions.push(str::from_utf8(&section[..6]).unwrap_or("000000"));
        // Start at 7 to skip over the ' ' separator
        filenames.push(str::from_utf8(&section[7..]).unwrap_or(""));
        // Read next 20 bytes to get the hash
        hashes.push(object_util::to_hex_string(
            &next_parse[section_end..section_end + 20],
        ));

        // Start where we left off reading the hash
        section_end += 20;
        next_parse = &next_parse[section_end..];
    }

    for i in 0..permissions.len() {
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

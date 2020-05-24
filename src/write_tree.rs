use super::object_util;
use sha1::Sha1;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::str;

// https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

/// Creates a tree object using the current index
///
/// Returns the name of the new tree object or an error message
pub fn write_tree(missing_ok: bool) -> std::string::String {
    let result = parse_index(missing_ok);
    match result {
        Ok(s) => s,
        Err(e) => format!("error: {}", e),
    }
}

fn parse_index(missing_ok: bool) -> Result<String, String> {
    let path = ".git/index";

    let mut file = fs::File::open(&path).expect("Could not read index file");

    let mut header = [0; 12];
    file.read_exact(&mut header).expect("Invalid index file");

    // Check magic number (DIRC)
    if header[..4].to_vec() != vec![0x44, 0x49, 0x52, 0x43] {
        return Err("invalid file type, expected DIRC".to_owned());
    }

    // Check verison number (2)
    // 3 and 4 are also valid versions, but not currently supported
    if header[4..8].to_vec() != vec![0x00, 0x00, 0x00, 0x02] {
        return Err("unsupported index version".to_owned());
    }

    let num_file = vec_to_int(&header[8..]);

    let mut tree_content = Vec::<u8>::new();

    for _ in 0..num_file {
        // Read past 6 32-bit (4-byte) info fields
        // ctime seconds, ctime nanosecond fractions, mtime seconds, mtime nanosecond fractions, dev (null on windows), ino (null on windows)
        file.read_exact(&mut [0; (4 * 6)])
            .expect("Invalid index format");
        let mut mode = [0; 4];
        file.read_exact(&mut mode).expect("Invalid index format");

        // read past 3 4-byte info fields
        // uid (null on windows), guid (null on windows), file size
        file.read_exact(&mut [0; (4 * 3)])
            .expect("Invalid index format");

        let mut hash = [0; 20];
        file.read_exact(&mut hash).expect("Invalid index format");

        let mut flags = [0; 2];
        file.read_exact(&mut flags).expect("Invalid index format");
        let filename_length = flags_to_length(&flags);

        let mut filename_buf = vec![0; filename_length as usize].into_boxed_slice();
        file.read_exact(&mut filename_buf)
            .expect("Invalid index format");

        // Read at least one NUL byte (and up to 8),
        // the filename ends with a NUL-terminator, and is padded to the nearest multiple of 8 bytes (for the entry)
        let bytes_read = 62; // bytes written for an entry
        let total_bytes = bytes_read + filename_length;
        let padding = 8 - (total_bytes % 8);
        let mut nul_buf = vec![0; padding as usize].into_boxed_slice();
        file.read_exact(&mut nul_buf).expect("Invalid index format");

        // First 2 bytes are always null, so we only need the last 2 bytes
        let permissions = mode_to_permissions(&mode[2..]);
        let filename = str::from_utf8(&filename_buf).unwrap();

        // Check that object exists in object database
        let hash_string = object_util::to_hex_string(&hash);
        let object_path = object_util::get_object_path(&hash_string);
        let exists = Path::new(&object_path).exists();

        if !exists && !missing_ok {
            let error = format!(
                "invalid object {} {} for {}\nfatal: write-tree: error building trees",
                permissions, hash_string, filename
            );
            return Err(error);
        }

        // permissions, space, the filename, null byte, the hex hash (20 bytes)
        // Write as bytes so the hash isn't mangled
        tree_content.append(&mut permissions.as_bytes().to_vec());
        tree_content.push(32u8); // space is 32 in ascii
        tree_content.append(&mut filename.as_bytes().to_vec());
        tree_content.push(0);
        tree_content.append(&mut hash.to_vec());
    }

    // A tree is a zlib compressed file of a header and a list of file information
    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    // File information is the permissions, space, the filename, null byte, the hex hash (20 bytes)

    let tree_header = format!("tree {}\0", tree_content.len());
    let mut store = Vec::from(tree_header.as_bytes());
    store.append(&mut tree_content);
    let hash = Sha1::from(&store).hexdigest();

    let store_buf = store.into_boxed_slice();

    object_util::write_object_file(&hash, &store_buf);

    Ok(hash)
}

/// Takes first 4 bytes and returns an unsigned int
fn vec_to_int(vec: &[u8]) -> u32 {
    (vec[0] as u32 * 1000) + (vec[1] as u32 * 100) + (vec[2] as u32 * 10) + (vec[3] as u32)
}

/// Takes last 2 bytes of mode and returns the file permissions
fn mode_to_permissions(vec: &[u8]) -> &str {
    // 4-bits: object type (reg, symlink, gitlink)
    // 3-bits: unused
    // 9-bit: permissions (755 or 644)
    // Since the only valid values for the first permission octal are 7 and 6 (111 or 110),
    // we only need the second byte to figure out the permissions
    if vec[1] == 0b10_100_100 {
        "644"
    } else {
        "755"
    }
}

/// Takes flag bytes and returns the length of the file name
fn flags_to_length(vec: &[u8]) -> u16 {
    // 1 bit assume valid
    // 1 bit extended (must be 0 in version 2)
    // 2 bit stage (during merge)
    // 12 bit name length if the length is less than 0xFF; otherwise 0xFF
    // Mask off the upper half of the the first byte so we can correctly cast it
    let high_bits = vec[0] & 0b0000_1111;
    // println!("high bits mask: {:08b} -> {:08b}", vec[0], high_bits);

    (high_bits as u16 * 10) + (vec[1] as u16)
}

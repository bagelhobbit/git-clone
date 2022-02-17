use super::object_util;
use sha1::Sha1;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::str;

const INDEX_PATH: &str = "gitrs/index";

#[allow(dead_code)]
pub struct IndexHeader {
    magic: String,
    version: u32,
    num_files: u32,
}

#[allow(dead_code)]
pub struct IndexFile {
    ctime: u32,
    ctime_fractions: u32,
    mtime: u32,
    mtime_fractions: u32,
    dev: Option<u32>,
    ino: Option<u32>,
    permissions: String,
    uid: Option<u32>,
    guid: Option<u32>,
    size: u32,
    object_hash: String,
    filename: String,
}

/// Parse the index file and writes it to the store as a tree object
///
/// Returns the hash of the resulting object
pub fn parse_index() -> Result<(IndexHeader, Vec<IndexFile>), String> {
    let mut file = fs::File::open(INDEX_PATH).expect("Could not read index file");

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

    let num_file = array_to_int(&header[8..]);

    // First 2 fields should be constant since we check for the magic 'number'
    // and only support 1 verison
    let index_header = IndexHeader {
        magic: String::from("DIRC"),
        version: 2,
        num_files: num_file,
    };

    let mut index_files = Vec::<IndexFile>::new();

    for _ in 0..num_file {
        // Read 6 32-bit (4-byte) info fields
        // ctime seconds, ctime nanosecond fractions, mtime seconds, mtime nanosecond fractions, dev (null on windows), ino (null on windows)
        let mut info_fields = [0; 24];
        file.read_exact(&mut info_fields)
            .expect("Invalid index format");

        let ctime = array_to_int(&info_fields[0..4]);
        let ctime_fractions = array_to_int(&info_fields[4..8]);
        let mtime = array_to_int(&info_fields[8..12]);
        let mtime_fractions = array_to_int(&info_fields[12..16]);
        let dev = array_to_int(&info_fields[16..20]);
        let ino = array_to_int(&info_fields[20..24]);

        let mut mode = [0; 4];
        file.read_exact(&mut mode).expect("Invalid index format");

        // read 3 4-byte info fields
        // uid (null on windows), guid (null on windows), file size
        let mut file_info = [0; 12];
        file.read_exact(&mut file_info)
            .expect("Invalid index format");

        let uid = array_to_int(&file_info[0..4]);
        let guid = array_to_int(&file_info[4..8]);
        let size = array_to_int(&file_info[8..12]);

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

        index_files.push(IndexFile {
            ctime,
            ctime_fractions,
            mtime,
            mtime_fractions,
            dev: if dev == 0 { None } else { Some(dev) },
            ino: if ino == 0 { None } else { Some(ino) },
            permissions: String::from(permissions),
            uid: if uid == 0 { None } else { Some(uid) },
            guid: if guid == 0 { None } else { Some(guid) },
            size,
            object_hash: object_util::to_hex_string(&hash),
            filename: String::from(filename),
        });
    }

    Ok((index_header, index_files))
}

/// Parses the index file and writes it to the store as a tree object
///
/// Returns the hash of the resulting object
pub fn write_index(missing_ok: bool) -> Result<String, String> {
    let (header, items) = parse_index()?;

    debug_assert_eq!(header.num_files as usize, items.len());

    let mut tree_content = Vec::<u8>::new();

    for file in items.iter() {
        // Check that object exists in object database
        let object_path = object_util::get_object_path(&file.object_hash);
        let exists = Path::new(&object_path).exists();

        if !exists && !missing_ok {
            //TODO: more generic error
            let error = format!(
                "invalid object {} {} for {}\nfatal: write-tree: error building trees",
                file.permissions, file.object_hash, file.filename
            );
            return Err(error);
        }

        // permissions, space, the filename, null byte, the hex hash (20 bytes)
        // Write as bytes so the hash isn't mangled
        tree_content.append(&mut file.permissions.as_bytes().to_vec());
        tree_content.push(32u8); // space is 32 in ascii
        tree_content.append(&mut file.filename.as_bytes().to_vec());
        tree_content.push(0);
        tree_content.append(&mut file.object_hash.as_bytes().to_vec());
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
fn array_to_int(array: &[u8]) -> u32 {
    ((array[0] as u32) << 12)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 4)
        + (array[3] as u32)
}

/// Takes last 2 bytes of permissions and returns the file permissions
fn mode_to_permissions(array: &[u8]) -> &str {
    // 4-bits: object type (reg, symlink, gitlink)
    // 3-bits: unused
    // 9-bit: permissions (755 or 644)
    // Since the only valid values for the first permission octal are 7 and 6 (111 or 110),
    // we only need the third byte to figure out the permissions
    // 1: type; 2: 000x; 3: xxxx; 4: xxxx
    // 644 => 110 100 100 => 1_1010_0100
    // 755 => 111 101 101 => 1_1110_1101
    if array[2] == 0b10_10 {
        "644"
    } else {
        "755"
    }
}

/// Takes flag bytes and returns the length of the file name
fn flags_to_length(array: &[u8]) -> u16 {
    // 1 bit assume valid
    // 1 bit extended (must be 0 in version 2)
    // 2 bit stage (during merge)
    // 12 bit name length if the length is less than 0xFF; otherwise 0xFF
    // Mask off the upper half of the the first byte so we can correctly cast it
    let low_bits = array[0] & 0b0000_1111;

    ((low_bits as u16) << 8) + (array[1] as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_to_permissons_rw() {
        let permissions = [0b1000u8, 0b000_1, 0b10_10, 0b0_100];
        assert_eq!("644", mode_to_permissions(&permissions))
    }

    #[test]
    fn test_mode_to_permissons_rwx() {
        let permissions = [0b1000u8, 0b000_1, 0b11_10, 0b1_101];
        assert_eq!("755", mode_to_permissions(&permissions))
    }

    #[test]
    fn test_array_to_int_size_4() {
        let array = [0x0u8, 0xB, 0xB, 0x8];
        assert_eq!(3000, array_to_int(&array));
    }

    #[test]
    fn test_array_to_int_oversized() {
        let array = [0x0u8, 0xB, 0xB, 0x8, 0xF, 0xF];
        assert_eq!(3000, array_to_int(&array))
    }

    #[test]
    // this should probably be a failure since the length is over 0xFF, but thats not being enforced here
    fn test_flags_to_length_long() {
        let array = [0b1111_1000, 0x88];
        assert_eq!(2184u16, flags_to_length(&array))
    }

    #[test]
    fn test_flags_to_length_average() {
        let array = [0b1111_0000, 0x0F];
        assert_eq!(15u16, flags_to_length(&array))
    }
}

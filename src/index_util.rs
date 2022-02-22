use super::object_util;
use sha1::Sha1;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::str;
use std::{fs, io};

const INDEX_PATH: &str = "gitrs/index";

#[derive(Debug)]
pub struct IndexHeader {
    pub magic: String,
    pub version: u32,
    pub num_files: u32,
}

#[derive(Debug, Clone)]
pub struct IndexFile {
    pub ctime: u32,
    pub ctime_fractions: u32,
    pub mtime: u32,
    pub mtime_fractions: u32,
    pub dev: Option<u32>,
    pub ino: Option<u32>,
    pub permissions: String,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub size: u32,
    pub object_hash: String,
    pub filename: String,
}

/// Parse the index file and return the index header and index files
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
        // Read 6 32-bit (24-byte) info fields
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

        let mut mode = [0; 2];
        file.read_exact(&mut mode).expect("Invalid index format");

        // read 3 32-bit (12 bytes) info fields
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
        let bytes_read = 60; // bytes written for an entry
        let total_bytes = bytes_read + filename_length;
        let padding = 8 - (total_bytes % 8);
        let mut nul_buf = vec![0; padding as usize].into_boxed_slice();
        file.read_exact(&mut nul_buf).expect("Invalid index format");

        let permissions = mode_to_permissions(&mode);
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
            gid: if guid == 0 { None } else { Some(guid) },
            size,
            object_hash: object_util::to_hex_string(&hash),
            filename: String::from(filename),
        });
    }

    Ok((index_header, index_files))
}

/// Writes the given index structs back to the index file
pub fn write_index(items: Vec<IndexFile>) -> io::Result<()> {
    // https://doc.rust-lang.org/stable/std/fs/struct.OpenOptions.html#method.truncate

    // Might want to create and replace the index file instead of overwriting it in case of errors
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(INDEX_PATH)
        .expect("Could not open index file");

    // Write magic number and version
    file.write_all(&[0x44, 0x49, 0x52, 0x43, 0x0, 0x0, 0x0, 0x02])?;

    let num_file = items.len() as u32;

    // Write number of files as 4 byte number
    file.write_all(&num_file.to_be_bytes())?;

    for item in items.iter() {
        // Write 6 32-bit (4-byte) info fields
        // ctime seconds, ctime nanosecond fractions, mtime seconds, mtime nanosecond fractions, dev (null on windows), ino (null on windows)
        file.write_all(&item.ctime.to_be_bytes())?;
        file.write_all(&item.ctime_fractions.to_be_bytes())?;
        file.write_all(&item.mtime.to_be_bytes())?;
        file.write_all(&item.mtime_fractions.to_be_bytes())?;
        match item.dev {
            Some(x) => file.write_all(&x.to_be_bytes())?,
            None => file.write_all(&[0, 0, 0, 0])?,
        };
        match item.ino {
            Some(x) => file.write_all(&x.to_be_bytes())?,
            None => file.write_all(&[0, 0, 0, 0])?,
        };

        // Write out mode
        // 4-bits: object type (regular (1000), symlink(1010), gitlink(1110))
        // 3-bits: unused
        // 9-bit: permissions (755 or 644)
        // assume regular file for now
        file.write_all(&[0b1000_0001])?;

        if item.permissions == "644" {
            file.write_all(&[0b1010_0100])?;
        } else {
            file.write_all(&[0b1110_1101])?;
        }

        // Write 3 4-byte info fields
        // uid (null on windows), guid (null on windows), file size
        match item.uid {
            Some(x) => file.write_all(&x.to_be_bytes())?,
            None => file.write_all(&[0, 0, 0, 0])?,
        };
        match item.gid {
            Some(x) => file.write_all(&x.to_be_bytes())?,
            None => file.write_all(&[0, 0, 0, 0])?,
        };
        file.write_all(&item.size.to_be_bytes())?;

        file.write_all(&hash_to_vec(&item.object_hash))?;

        let filename_length = item.filename.len();
        debug_assert!(filename_length <= 255);

        // write flag bytes
        // 1 bit assume valid
        // 1 bit extended (must be 0 in version 2)
        // 2 bit stage (during merge)
        // 12 bit name length if the length is less than 0xFF; otherwise 0xFF
        file.write_all(&[0b1000_0000, (filename_length as u8)])?;

        file.write_all(item.filename.as_bytes())?;

        // Write at least one NUL byte (and up to 8),
        // The filename ends with a NUL-terminator, and is padded to the nearest multiple of 8 bytes (for the entry)
        let bytes_written = 60; // bytes written for an entry
        let total_bytes = bytes_written + filename_length;
        let padding = 8 - (total_bytes % 8);
        let nul_buf = vec![0; padding as usize];
        file.write_all(&nul_buf)?;
    }

    Ok(())
}

/// Parses the index file and writes it to the store as a tree object
///
/// Returns the hash of the resulting object
pub fn write_index_to_tree(missing_ok: bool) -> Result<String, String> {
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
        tree_content.append(&mut "100".as_bytes().to_vec());
        tree_content.append(&mut file.permissions.as_bytes().to_vec());
        tree_content.push(32u8); // space is 32 in ascii
        tree_content.append(&mut file.filename.as_bytes().to_vec());
        tree_content.push(0);
        tree_content.append(&mut hash_to_vec(&file.object_hash));
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
    if array[1] == 0b1010_0100 {
        "644"
    } else {
        "755"
    }
}

/// Takes flag bytes and returns the length of the file name
fn flags_to_length(array: &[u8]) -> u8 {
    // 1 bit assume valid
    // 1 bit extended (must be 0 in version 2)
    // 2 bit stage (during merge)
    // 12 bit name length, if the length is less than 0xFF; otherwise 0xFF
    // Mask off the upper half of the the first byte so we can correctly cast it
    array[1]
}

/// Convert an object hash to a vector for usable in the index file
fn hash_to_vec(hash: &str) -> Vec<u8> {
    let mut converted: Vec<u8> = Vec::new();
    for char in hash.chars() {
        // assume only hex compatible chars
        let x = match char {
            '0' => 0x0u8,
            '1' => 0x1,
            '2' => 0x2,
            '3' => 0x3,
            '4' => 0x4,
            '5' => 0x5,
            '6' => 0x6,
            '7' => 0x7,
            '8' => 0x8,
            '9' => 0x9,
            'A' | 'a' => 0xA,
            'B' | 'b' => 0xB,
            'C' | 'c' => 0xC,
            'D' | 'd' => 0xD,
            'E' | 'e' => 0xE,
            'F' | 'f' => 0xF,
            _ => panic!("Unsupported character"),
        };
        converted.push(x);
    }

    let chunks = converted.chunks(2);
    let mut result: Vec<u8> = Vec::new();
    for chunk in chunks {
        result.push((chunk[0] << 4) + chunk[1]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_to_permissons_rw() {
        let permissions = [0b1000_0001, 0b1010_0100];
        assert_eq!("644", mode_to_permissions(&permissions))
    }

    #[test]
    fn test_mode_to_permissons_rwx() {
        let permissions = [0b1000_0001, 0b1110_1101];
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
    fn test_flags_to_length_max() {
        let array = [0b1000_0000, 0xFF];
        assert_eq!(255u8, flags_to_length(&array))
    }

    #[test]
    fn test_flags_to_length_average() {
        let array = [0b1000_0000, 0x0F];
        assert_eq!(15u8, flags_to_length(&array))
    }

    #[test]
    fn test_hash_to_vec() {
        let hash = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
        let bytes = vec![
            0x4b, 0x82, 0x5d, 0xc6, 0x42, 0xcb, 0x6e, 0xb9, 0xa0, 0x60, 0xe5, 0x4b, 0xf8, 0xd6,
            0x92, 0x88, 0xfb, 0xee, 0x49, 0x04,
        ];
        assert_eq!(bytes, hash_to_vec(hash));
    }
}

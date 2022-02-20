use std::fs;
use std::time::SystemTime;

use super::hash_object::write_hash_object;
use super::index_util;
use super::index_util::IndexFile;

// https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

/// Register file contents in the working tree to the index
///
/// Ignores new files
#[allow(dead_code)]
pub fn update_index() -> Result<String, String> {
    todo!()
}

/// Add specified file to the index
pub fn add_to_index(filepath: &str) -> Result<String, String> {
    let (_header, mut items) = index_util::parse_index()?;
    // call 'update_index' to update all other files?

    let hash = write_hash_object(filepath);

    let metadata = match fs::metadata(filepath) {
        Ok(m) => m,
        Err(e) => return Err(e.to_string()),
    };

    let new_item = IndexFile {
        ctime: metadata
            .created()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
        ctime_fractions: 0,
        mtime: metadata
            .modified()
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
        mtime_fractions: 0,
        dev: None,
        ino: None,
        permissions: "644".to_string(),
        uid: None,
        guid: None,
        size: metadata.len() as u32,
        object_hash: hash,
        filename: filepath.to_string(),
    };
    items.push(new_item);

    match index_util::write_index(items) {
        Ok(_) => Ok("".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Remove specified file from the index
#[allow(dead_code)]
#[allow(unused_variables)]
pub fn remove_from_index(force: bool) -> Result<String, String> {
    todo!()
}

use std::fs;
use std::time::{Duration, SystemTime};

use super::hash_object::write_hash_object;
use super::index_util;
use super::index_util::IndexFile;

// https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

/// Register file contents in the working tree to the index
///
/// Ignores new files
pub fn update_index() -> Result<(), String> {
    let (_header, items) = index_util::parse_index()?;

    let updated_items = update_index_items(items);

    match index_util::write_index(updated_items) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Add specified file to the index and update existing items
///
/// Will fail for duplicate items
pub fn add_to_index(filepath: &str) -> Result<(), String> {
    let (_header, mut items) = index_util::parse_index()?;

    // filepath should be normalized to avoid false negatives
    for item in items.iter() {
        if filepath == item.filename {
            return Err("This file already exists in the index".to_string());
        }
    }

    items = update_index_items(items);

    let object_hash = write_hash_object(filepath);

    let mut ctime = 0u32;
    let mut mtime = 0u32;
    let mut size = 0u32;

    if let Ok(metadata) = fs::metadata(filepath) {
        ctime = metadata
            .created()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::new(0, 0))
            .as_secs() as u32;
        mtime = metadata
            .modified()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::new(0, 0))
            .as_secs() as u32;
        size = metadata.len() as u32;
    }

    let new_item = IndexFile {
        ctime,
        ctime_fractions: 0,
        mtime,
        mtime_fractions: 0,
        dev: None,
        ino: None,
        permissions: "644".to_string(),
        uid: None,
        gid: None,
        size,
        object_hash,
        filename: filepath.to_string(),
    };
    items.push(new_item);

    match index_util::write_index(items) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Remove specified file from the index
#[allow(dead_code)]
#[allow(unused_variables)]
pub fn remove_from_index(force: bool) -> Result<(), String> {
    todo!()
}

/// Update [`IndexFile`]'s hash and file properties
fn update_index_items(items: Vec<IndexFile>) -> Vec<IndexFile> {
    let mut updated_items: Vec<IndexFile> = Vec::new();
    for item in items.iter() {
        let object_hash = write_hash_object(&item.filename);

        let mut mtime = 0u32;
        let mut size = 0u32;

        // metadata also has dev, ino, uid, and gid fields but since windows doesn't use them
        // I'm ignoring it for now
        if let Ok(metadata) = fs::metadata(&item.filename) {
            mtime = metadata
                .modified()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs() as u32;

            size = metadata.len() as u32;
        }

        let updated = IndexFile {
            mtime,
            size,
            object_hash,
            ..item.clone()
        };
        updated_items.push(updated);
    }
    updated_items
}

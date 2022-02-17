use super::index_util;

// https://github.com/git/git/blob/master/Documentation/technical/index-format.txt

/// Creates a tree object using the current index
///
/// Returns the name of the new tree object or an error message
pub fn write_tree(missing_ok: bool) -> std::string::String {
    let result = index_util::write_index(missing_ok);
    match result {
        Ok(s) => s,
        Err(e) => format!("error: {}", e),
    }
}

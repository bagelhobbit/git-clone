use super::ls_tree;
use super::object_util;
use super::object_util::Object;
use std::str;
use std::string::String;

#[derive(Debug)]
pub enum CatFlags {
    Print,
    Type,
    Size,
}

// https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

/// Returns various properties of the passed object.
///
/// Current properties are file contents, header type, and content size.
///
/// # Arguments
///
/// * `flag` - Controls which property will be returned
/// * `object_hash` - The object in the object database to read
pub fn cat_file(flag: CatFlags, object_hash: &str) -> String {
    let git_object = object_util::read_object_file(&object_hash);
    let decoded = object_util::decode_object(git_object);

    // TODO: move reading/parsing of header into utils? Use header struct instead of methods?
    let mut split = decoded.split(|num| num == &0u8);
    let header = split.next().unwrap();
    let object_type = object_util::get_header_type(&header);

    match flag {
        CatFlags::Print => {
            // Assume the file has valid contents
            if object_type == Object::Tree {
                return ls_tree::format_tree(&decoded);
            } else {
                let content = split.next().unwrap();
                return str::from_utf8(&content).unwrap().to_owned();
            }
        }
        CatFlags::Type => return format!("{}\n", object_type),
        CatFlags::Size => return format!("{}\n", object_util::get_header_size(&header)),
    }
}

// TODO: support other methods to specify commits,
// for example by short hash or refname
// https://git-scm.com/docs/gitrevisions
/// Parse arguments for `cat_file`
pub fn parse_args(flag: &str, hash: &str) -> Result<(CatFlags, String), String> {
    let cat_flag = match flag {
        _ if flag == "-p" => Ok(CatFlags::Print),
        _ if flag == "-t" => Ok(CatFlags::Type),
        _ if flag == "-s" => Ok(CatFlags::Size),
        _ => Err(format!("{} is not recognized as a valid option", flag)),
    };

    return Ok((cat_flag?, hash.to_owned()));
}

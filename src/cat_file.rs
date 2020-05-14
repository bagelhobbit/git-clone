use flate2::read::ZlibDecoder;
use std::fmt;
use std::fs;
use std::io::Read;
use std::str;

#[derive(Debug)]
pub enum CatFlags {
    Print,
    Type,
    Size,
}

#[derive(Debug)]
enum Object {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Object::Blob => write!(f, "blob"),
            Object::Tree => write!(f, "tree"),
            Object::Commit => write!(f, "commit"),
        }
    }
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
pub fn cat_file(flag: CatFlags, object_hash: &str) -> std::string::String {
    let object_dir = "gitrs/objects/";
    // The first 2 characters of the hash is the directory the object is stored in
    let hash_dir = &object_hash[..2];
    // The remaing characters are the filename
    let filename = &object_hash[2..];
    let path = format!("{}{}/{}", &object_dir, &hash_dir, &filename);

    let git_object = fs::read(&path).expect("Could not read object file");
    let mut decoder = ZlibDecoder::new(&git_object[..]);
    let mut decompressed: Vec<u8> = Vec::new();

    // Assume file is valid
    decoder.read_to_end(&mut decompressed).unwrap();
    // println!("{:?}", decompressed);

    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    let mut split = decompressed.split(|num| num == &0u8);
    let header = split.next().unwrap();
    let object_type = get_header_type(&header);

    match flag {
        CatFlags::Print => {
            // Assume the file has valid contents
            // TODO: check header types

            // A blob is a zlib compressed file of a header and the file contents
            // A tree is a zlib compressed file of a header and  ??? a list of modes and files (null terminated lines?) TODO: read all lines, see output.txt
            // A commit object is a zlib compressed file of a header and the commit contents
            let content = split.next().unwrap();
            return str::from_utf8(&content).unwrap().to_owned();
        }
        CatFlags::Type => return format!("{}\n", object_type),
        CatFlags::Size => {
            // space (' ') is 32(dec) in ascii
            let mut split = header.split(|c| c == &32u8);
            let _ = split.next();
            let header_size = split.next().unwrap();
            return format!("{}\n", str::from_utf8(&header_size).unwrap());
        }
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

fn get_header_type(header: &[u8]) -> Object {
    // space (' ') is 32(dec) in ascii
    let mut split = header.split(|c| c == &32u8);
    let header_type = split.next().expect("Invalid header");

    let tmp = str::from_utf8(&header_type).unwrap();

    match tmp {
        _ if tmp == "blob" => return Object::Blob,
        _ if tmp == "tree" => return Object::Tree,
        _ if tmp == "commit" => return Object::Commit,
        _ => panic!("Invalid header"),
    };
}

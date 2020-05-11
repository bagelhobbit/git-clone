use std::fs;
use std::io::Read;
use std::str;
use flate2::read::ZlibDecoder;

// TODO: support other methods of specify commits,
// for example by short hash or refname
// https://git-scm.com/docs/gitrevisions
pub fn cat_file(object_hash: &str) {
    let object_dir = ".git/objects/";
    // The first 2 characters of the hash is the directory the object is stored in
    let hash_dir = &object_hash[..2];
    // The remaing characters are the filename
    let filename = &object_hash[2..];
    let path = format!("{}{}/{}", &object_dir, &hash_dir, &filename);

    // A blob is a zlib compressed file of a header and the file contents
    // A tree is ...
    // A commit object is ...
    let git_object = fs::read(&path).expect("Could not read object file");

    let mut decoder = ZlibDecoder::new(&git_object[..]);
    let mut decompressed = String::new();
    // Assume file is valid
    decoder.read_to_string(&mut decompressed).unwrap();

    // An object header is the type of object, a space, the size of the contents in bytes, then a null byte
    // Assuming the object is a blob
    // TODO: check header types
    let mut split = decompressed.as_bytes().split(|num| num == &0u8);
    let _header = split.next().unwrap();
    let content = split.next().unwrap();

    println!("{:?}", &_header);
    println!("{}", str::from_utf8(&content).unwrap());
}

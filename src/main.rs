use std::env;

mod cat_file;
mod hash_object;
mod init;
mod ls_tree;
mod object_util;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = &args[1];

    match command {
        _ if command == "init" => {
            init::create_git_dir().unwrap();
            println!(
                "Initialized empty Git repository in {}",
                env::current_dir().unwrap().to_str().unwrap()
            )
        }
        _ if command == "cat-file" => {
            if args.len() >= 4 {
                let params = cat_file::parse_args(&args[2], &args[3]);
                match params {
                    // cat_file output has newlines included, so don't reprint them here
                    Ok((flag, hash)) => print!("{}", cat_file::cat_file(flag, &hash)),
                    Err(e) => println!("{}", e),
                };
            } else {
                println!("usage: cat-file (-t | -s | -p) <object>\n");
                println!("    -t\t\tshow the object type");
                println!("    -s\t\tshow the object size");
                println!("    -p\t\tPretty print object's contents");
            }
        }
        _ if command == "hash-object" => {
            // TODO: support `-t <type>`
            if args.len() >= 4 {
                if args[2] == "-w" {
                    println!("{}", hash_object::write_hash_object(&args[3]))
                }
            } else if args.len() >= 3 {
                println!("{}", hash_object::hash_object(&args[2]))
            } else {
                println!("usage: hash-object [-w] <file>\n");
                println!("    -w\t\twrite the object into the object database");
            }
        }
        _ if command == "ls-tree" => {
            // TODO: support some flags, ex. -d, -r, -t
            if args.len() >= 3 {
                print!("{}", ls_tree::ls_tree(&args[2]))
            } else {
                println!("usage: ls-tree <object>\n");
            }
        }
        _ => println!("{} is not recognized as a valid command", command),
    }
}

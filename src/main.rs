use std::env;

use update_index::update_index;

mod cat_file;
mod hash_object;
mod index_util;
mod init;
mod ls_tree;
mod object_util;
mod update_index;
mod write_tree;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("usage: gitrs <command> [<args>]");
        return;
    }

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
            if args.len() >= 4 {
                if args[2] == "-w" {
                    println!("{}", hash_object::write_hash_object(&args[3]))
                }
            } else if args.len() >= 3 {
                println!("{}", hash_object::generate_hash(&args[2]))
            } else {
                println!("usage: hash-object [-w] <file>\n");
                println!("    -w\t\twrite the object into the object database");
            }
        }
        _ if command == "ls-tree" => {
            if args.len() >= 3 {
                print!("{}", ls_tree::ls_tree(&args[2]))
            } else {
                println!("usage: ls-tree <object>\n");
            }
        }
        _ if command == "write-tree" => {
            if args.len() >= 3 {
                if args[2] == "--missing-ok" {
                    println!("{}", write_tree::write_tree(true));
                } else {
                    println!("usage: write-tree [--missing-ok]\n");
                    println!("    --missing-ok\t\tallow missing objects");
                }
            } else {
                println!("{}", write_tree::write_tree(false));
            }
        }
        _ if command == "update-index" => {
            if args.len() >= 4 {
                if args[2] == "--add" {
                    if let Err(s) = update_index::add_to_index(&args[3]) {
                        println!("{}", s);
                    }
                }
            } else if let Err(s) = update_index() {
                println!("{}", s);
            }
        }
        _ => println!("{} is not recognized as a valid command", command),
    }
}

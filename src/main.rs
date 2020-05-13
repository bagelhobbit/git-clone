use std::env;

mod init;
mod objects;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

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
            // TODO: support (at least) -p, -t, -s
            if args.len() >= 4 {
                let params = objects::parse_args(&args[2], &args[3]);
                match params {
                    Ok((flag, hash)) => objects::cat_file(flag, &hash),
                    Err(e) => println!("{}", e),
                };
            } else {
                println!("usage: cat-file (-t | -s | -p) <object>\n");
                println!("    -t\t\tshow the object type");
                println!("    -s\t\tshow the object size");
                println!("    -p\t\tPretty print object's contents");
            }
        }
        _ => println!("{} is not recognized as a valid command", command),
    }
}

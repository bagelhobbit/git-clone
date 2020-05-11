use std::env;

mod init;
mod objects;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let command = &args[1];

    match command {
        _ if command == "init" => {
            println!("Creating init directory");
            init::create_git_dir().unwrap();
            println!(
                "Initialized empty Git repository in {}",
                env::current_dir().unwrap().to_str().unwrap()
            )
        }
        _ if command == "cat-file" => {
            // assume `-p` for now
            // TODO: support (at least) -p, -t, -s
            if args.len() >= 3 {
                objects::cat_file(&args[2]);
            } else {
                // TODO: print usage?
                println!("Please provide an object id")
            }
        }
        _ => println!("{} is not recognized as a valid command", command),
    }
}

use std::fs;
use std::io;

pub fn create_git_dir() -> io::Result<()> {
    let dir = "gitrs/";
    match fs::create_dir(dir) {
        Ok(_) => (),
        Err(e) => {
            if e.kind() == io::ErrorKind::AlreadyExists {
                println!("Directory has already been initialized. Nothing left to do");
                ()
            } else {
                return Err(e);
            }
        }
    }
    // See docs/InitFiles.md for a general description of the file structure being created
    fs::create_dir(format!("{}hooks", dir))?;
    fs::create_dir(format!("{}info", dir))?;
    fs::create_dir(format!("{}objects", dir))?;
    fs::create_dir(format!("{}objects/info", dir))?;
    fs::create_dir(format!("{}objects/pack", dir))?;
    fs::create_dir(format!("{}refs", dir))?;
    fs::create_dir(format!("{}refs/heads", dir))?;
    fs::create_dir(format!("{}refs/tags", dir))?;
    create_and_copy_to_file("initFiles/exclude", &format!("{}info/exclude", dir))?;
    create_and_copy_to_file("initFiles/config", &format!("{}config", dir))?;
    fs::write(
        format!("{}description", dir),
        "Unnamed repository; edit this file 'description' to name the repository.\n",
    )?;
    fs::write(format!("{}HEAD", dir), "ref: refs/heads/master\n")?;
    Ok(())
}
fn create_and_copy_to_file(from: &str, to: &str) -> io::Result<()> {
    fs::File::create(to)?;
    fs::copy(from, to)?;
    Ok(())
}

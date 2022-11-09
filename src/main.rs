use std::fs;
use std::path::Path;

const DIR: &str = env!("DB_CONNECTIONS_DIR");

/// Recursively traverse from starting directory.
fn traverse(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            traverse(&entry.path())?
        } else if let Some(v) = entry.path().extension() {
            if v == "txt" {
                println!("{:?}", entry.path());
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let dir = Path::new(DIR);
    if dir.is_dir() {
        if let Err(e) = traverse(dir) {
            // for now, just show errors
            // will need to handle/explicitly ignore later
            eprintln!("Error: {e}")
        }
    } else {
        eprintln!("Cannot find directory {:?}", dir);
    }
    Ok(())
}

use std::fs;
use std::path::Path;

const DIR: &str = "/home/kris";

/// Recursively traverse from starting directory.
fn traverse(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        // will need to ignore/track errors (such as PermissionDenied)
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
        traverse(dir)?;
    } else {
        eprintln!("Cannot find directory {:?}", dir);
    }

    Ok(())
}

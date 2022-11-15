use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use csv::Writer;
use serde::{Deserialize, Serialize};

const DIR: &str = env!("DB_CONNECTIONS_DIR");

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Add {
    name: String,
    #[serde(rename = "connectionString")]
    connection_string: String,
    provider: Option<String>,
    #[serde(rename = "providerName")]
    provider_name: Option<String>,
}

#[derive(Debug)]
struct ConnectionString {
    data_source: Option<String>,
    user_id: Option<String>,
    password: Option<String>,
    path: Option<String>,
}

impl ConnectionString {
    /// Parse connection string
    fn new(conn_str: String, filepath: String) -> Result<Self, String> {
        let mut connection_string = Self {
            data_source: None,
            user_id: None,
            password: None,
            path: None,
        };

        for pair in conn_str.split(';') {
            if let Some(v) = pair.trim().split_once('=') {
                if v.0.trim() == "Data Source" {
                    connection_string.data_source = Some(v.1.trim().to_string());
                }
                if v.0.trim() == "User Id" {
                    connection_string.user_id = Some(v.1.trim().to_string());
                }
                if v.0.trim() == "Password" {
                    connection_string.password = Some(v.1.trim().to_string());
                }
            }
            connection_string.path = Some(filepath.clone());
        }
        if connection_string.data_source.is_none() || connection_string.user_id.is_none() {
            return Err("Notice: no data source or user id in string".to_string());
        }
        Ok(connection_string)
    }
}

fn extract(f: &Path) -> Vec<ConnectionString> {
    // let config = ParserConfig::new().trim_whitespace(true);

    let mut connection_strings = vec![];

    let content = fs::read_to_string(f).unwrap();
    for line in content.lines() {
        if line.trim().starts_with('<') && line.contains("provider") {
            match serde_xml_rs::from_str::<Add>(line) {
                Ok(v) => {
                    if let Ok(v) = ConnectionString::new(
                        v.connection_string.clone(),
                        f.to_string_lossy().to_string(),
                    ) {
                        connection_strings.push(v)
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
    }
    connection_strings
}

// Get files matching extensions we want
fn traverse(dir: PathBuf, mut files: Vec<PathBuf>) -> std::io::Result<Vec<PathBuf>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files = traverse(entry.path(), files)?
        } else if let Some(v) = entry.path().extension() {
            if let Some(v) = v.to_str() {
                if ["config", "aspx", "asp"].contains(&v) {
                    files.push(entry.path().clone());
                }
            }
        }
    }
    Ok(files)
}

fn main() -> std::io::Result<()> {
    let dir = Path::new(DIR);
    let files = vec![];
    let mut connection_strings = vec![];

    if dir.is_dir() {
        if let Ok(v) = traverse(dir.to_path_buf(), files) {
            for file in v {
                connection_strings.extend(extract(&file));
            }
        }
    }

    let mut wtr = Writer::from_path("output.csv")?;
    wtr.write_record(["path", "data source", "user id", "password"])?;
    for cs in connection_strings {
        wtr.write_record(&[
            cs.path.unwrap(),
            cs.data_source.unwrap(),
            cs.user_id.unwrap(),
            cs.password.unwrap(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // test extraction of connection string from line

    // test parsing of connection string
    #[test]
    fn conn_str_created_successfully() {
        let raw_cs = "Data Source=dvrpcdb2; User Id=NETS; Password=something;".to_string();
        let cs = ConnectionString::new(raw_cs, "some filepath".to_string());
        assert!(matches!(cs, Ok(_)));
        let cs = cs.unwrap();
        assert_eq!(cs.data_source, Some("dvrpcdb2".to_string()));
        assert_eq!(cs.user_id, Some("NETS".to_string()));
        assert_eq!(cs.password, Some("something".to_string()))
    }
}

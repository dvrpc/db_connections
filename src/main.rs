use std::fs;
use std::path::Path;

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
}

impl ConnectionString {
    /// Parse connection string
    fn new(conn_str: String) -> Result<Self, String> {
        let mut connection_string = Self {
            data_source: None,
            user_id: None,
            password: None,
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
        }
        if connection_string.data_source.is_none() || connection_string.user_id.is_none() {
            return Err("Notice: no data source or user id in string".to_string());
        }
        Ok(connection_string)
    }
}

/// Use xml parser to get connection string
fn extract(f: &Path) {
    // let config = ParserConfig::new().trim_whitespace(true);

    let content = fs::read_to_string(f).unwrap();
    for line in content.lines() {
        if line.trim().starts_with('<') && line.contains("provider") {
            match serde_xml_rs::from_str::<Add>(line) {
                Ok(v) => {
                    let cs = ConnectionString::new(v.connection_string.clone());
                    println!("{:?}: {:?}", f, cs);
                }
                Err(e) => println!("{e}"),
            }
        }
    }
}

/// Recursively traverse from starting directory.
fn traverse(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            traverse(&entry.path())?
        } else if let Some(v) = entry.path().extension() {
            if let Some(v) = v.to_str() {
                if ["config", "aspx", "asp"].contains(&v) {
                    extract(&entry.path());
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    // test extraction of connection string from line

    // test parsing of connection string
    #[test]
    fn conn_str_created_successfully() {
        let raw_cs = "Data Source=dvrpcdb2; User Id=NETS; Password=something;".to_string();
        let cs = ConnectionString::new(raw_cs);
        assert!(matches!(cs, Ok(_)));
        let cs = cs.unwrap();
        assert_eq!(cs.data_source, Some("dvrpcdb2".to_string()));
        assert_eq!(cs.user_id, Some("NETS".to_string()));
        assert_eq!(cs.password, Some("something".to_string()))
    }
}

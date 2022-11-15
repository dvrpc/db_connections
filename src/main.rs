use std::fs;
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
struct Connection {
    path: Option<String>,
    data_source: Option<String>,
    user_id: Option<String>,
    provider: Option<String>,
}

impl Connection {
    fn new(path: String, conn_str: String, provider: String) -> Result<Self, String> {
        let mut connection = Self {
            path: None,
            data_source: None,
            user_id: None,
            provider: None,
        };

        for pair in conn_str.split(';') {
            if let Some(v) = pair.trim().split_once('=') {
                if v.0.trim() == "Data Source" {
                    connection.data_source = Some(v.1.trim().to_string());
                }
                if v.0.trim() == "User Id" {
                    connection.user_id = Some(v.1.trim().to_string());
                }
            }
        }
        connection.path = Some(path);
        connection.provider = Some(provider);

        if connection.data_source.is_none() || connection.user_id.is_none() {
            return Err("Notice: no data source or user id in string".to_string());
        }
        Ok(connection)
    }
}

fn extract(f: &Path) -> Vec<Connection> {
    // let config = ParserConfig::new().trim_whitespace(true);

    let mut connections = vec![];

    let content = fs::read_to_string(f).unwrap();
    for line in content.lines() {
        if line.trim().starts_with('<') && line.contains("provider") {
            match serde_xml_rs::from_str::<Add>(line) {
                Ok(v) => {
                    let provider = if v.provider.is_some() {
                        v.provider.unwrap()
                    } else if v.provider_name.is_some() {
                        v.provider_name.unwrap()
                    } else {
                        // let this pass but point it out
                        "Unrecognized provider format".to_string()
                    };

                    if let Ok(v) = Connection::new(
                        f.to_string_lossy().to_string(),
                        v.connection_string.clone(),
                        provider,
                    ) {
                        connections.push(v)
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
    }
    connections
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
    // crawl files and extract db connections
    let dir = Path::new(DIR);
    let files = vec![];
    let mut connections = vec![];

    if dir.is_dir() {
        if let Ok(v) = traverse(dir.to_path_buf(), files) {
            for file in v {
                connections.extend(extract(&file));
            }
        }
    }

    // write to CSV
    let mut wtr = Writer::from_path("output.csv")?;
    wtr.write_record(["path", "data source", "user id", "provider"])?;
    for cs in connections {
        wtr.write_record(&[
            cs.path.unwrap(),
            cs.data_source.unwrap(),
            cs.user_id.unwrap(),
            cs.provider.unwrap(),
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
    fn conn_created_successfully() {
        let raw_cs = "Data Source=dvrpcdb2; User Id=NETS; Password=something;".to_string();
        let cs = Connection::new(
            "some filepath".to_string(),
            raw_cs,
            "some provider".to_string(),
        );
        assert!(matches!(cs, Ok(_)));
        let cs = cs.unwrap();
        assert_eq!(cs.data_source, Some("dvrpcdb2".to_string()));
        assert_eq!(cs.user_id, Some("NETS".to_string()));
        assert_eq!(cs.provider, Some("some provider".to_string()))
    }
}

use std::fs;
use std::io::Write;
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
            return Err("No data source or user id in string".to_string());
        }
        Ok(connection)
    }
}

/// Extract connection string from xml, return tuple of successes and errors
fn extract(f: &Path) -> (Vec<Connection>, Vec<String>) {
    let mut connections = vec![];
    let mut errors = vec![];

    let content = fs::read_to_string(f).unwrap();
    for line in content.lines() {
        if line.trim().starts_with('<') && line.contains("provider") {
            match serde_xml_rs::from_str::<Add>(line) {
                Ok(v) => {
                    if v.provider.is_none() && v.provider_name.is_none() {
                        errors.push(format!("{:?}: Provider not found.", f));
                        continue;
                    }
                    let provider = if v.provider.is_some() {
                        v.provider.unwrap()
                    } else {
                        v.provider_name.unwrap()
                    };

                    match Connection::new(
                        f.to_string_lossy().to_string(),
                        v.connection_string.clone(),
                        provider,
                    ) {
                        Ok(v) => connections.push(v),
                        Err(e) => errors.push(format!("{:?}: {}", f, e)),
                    }
                }
                Err(e) => errors.push(format!("{:?}: {}", f, e)),
            }
        }
    }
    (connections, errors)
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
    let mut errors = vec![];

    if dir.is_dir() {
        if let Ok(v) = traverse(dir.to_path_buf(), files) {
            for file in v {
                let (c, e) = extract(&file);
                connections.extend(c);
                errors.extend(e);
            }
        }
    }

    // write to connections to CSV file
    if !connections.is_empty() {
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
    }

    // write errors to text file
    if !errors.is_empty() {
        let mut error_file = fs::File::create("errors.txt")?;
        for error in errors {
            writeln!(error_file, "{}", error)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // test extraction of connection string from line

    // test parsing of connection string
    #[test]
    fn conn_created_successfully() {
        let c = Connection::new(
            "path".to_string(),
            "Data Source=db2; User Id=NETS; Password=something;".to_string(),
            "provider".to_string(),
        );
        assert!(c.is_ok());
        let c = c.unwrap();
        assert_eq!(c.data_source, Some("db2".to_string()));
        assert_eq!(c.user_id, Some("NETS".to_string()));
        assert_eq!(c.provider, Some("provider".to_string()))
    }

    #[test]
    fn conn_created_successfully_no_password() {
        let c = Connection::new(
            "path".to_string(),
            "Data Source=db2; User Id=NETS;".to_string(),
            "provider".to_string(),
        );
        assert!(c.is_ok());
        let c = c.unwrap();
        assert_eq!(c.data_source, Some("db2".to_string()));
        assert_eq!(c.user_id, Some("NETS".to_string()));
        assert_eq!(c.provider, Some("provider".to_string()))
    }

    #[test]
    fn new_connection_errs_no_data_source() {
        assert!(Connection::new(
            "path".to_string(),
            "User Id=NETS".to_string(),
            "provider".to_string(),
        )
        .is_err())
    }
    #[test]
    fn new_connection_ok_data_source_exists_but_empty() {
        let c = Connection::new(
            "path".to_string(),
            "Data Source=; User Id=NETS;".to_string(),
            "provider".to_string(),
        );
        assert!(c.is_ok());
        assert_eq!(c.unwrap().data_source.unwrap(), "".to_string());
    }
    #[test]
    fn new_connection_errs_no_user_id() {
        assert!(Connection::new(
            "path".to_string(),
            "Data Source=db2".to_string(),
            "provider".to_string(),
        )
        .is_err())
    }
    #[test]
    fn new_connection_ok_user_id_exists_but_empty() {
        let c = Connection::new(
            "path".to_string(),
            "Data Source=db2; User Id= ;".to_string(),
            "provider".to_string(),
        );
        assert!(c.is_ok());
        assert_eq!(c.unwrap().user_id.unwrap(), "".to_string());
    }
}

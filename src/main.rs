use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use csv::Writer;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AspNet {
    name: String,
    #[serde(rename = "connectionString")]
    connection_string: String,
    provider: Option<String>,
    #[serde(rename = "providerName")]
    provider_name: Option<String>,
}

#[derive(Debug, Clone)]
struct Connection {
    path: String,
    data_source: String,
    user_id: String,
    provider: String,
}

/// Get files matching extensions we want.
fn get_files(dir: PathBuf, mut files: Vec<PathBuf>) -> std::io::Result<Vec<PathBuf>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files = get_files(entry.path(), files)?
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

/// Iterate through files and extract Connections.
fn extract_connections_from_files(files: Vec<PathBuf>) -> (Vec<Connection>, Vec<String>) {
    let mut connections = vec![];
    let mut errors = vec![];

    for file in files {
        let content = fs::read_to_string(file.clone()).unwrap();
        let extension = file.extension().unwrap().to_str().unwrap();

        if extension == "config" || extension == "aspx" {
            // read file to string and then capture regular expression matches
            // (anything in angle brackets)
            let re = Regex::new(r"(?s)<.*?>").unwrap();
            let results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            for result in results {
                match extract_from_asp_net(result, &file) {
                    Ok(None) => (),
                    Ok(Some(v)) => connections.push(v),
                    Err(e) => errors.push(e),
                }
            }
        } else if extension == "asp" {
            // read file to string and then capture regular expression matches
            // (anything in double quotes)
            let re = Regex::new(r#"(?s)".*?""#).unwrap();
            let results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            for result in results {
                match extract_from_asp_classic(result, &file) {
                    Ok(None) => (),
                    Ok(Some(v)) => connections.push(v),
                    Err(e) => errors.push(e),
                }
            }
        }
    }
    (connections, errors)
}

/// Extract Connection from element string in ASP.NET format
fn extract_from_asp_net(element: &str, file: &Path) -> Result<Option<Connection>, String> {
    if !element.contains("connectionString") {
        return Ok(None);
    }

    match serde_xml_rs::from_str::<AspNet>(element) {
        Ok(v) => {
            if v.provider.is_none() & v.provider_name.is_none() {
                return Err(format!("{:?}: Provider not found.", file));
            }

            let provider = if v.provider.is_none() {
                v.provider_name.unwrap()
            } else {
                v.provider.unwrap()
            };

            let mut data_source = None;
            let mut user_id = None;

            for pair in v.connection_string.split(';') {
                if let Some(v) = pair.trim().split_once('=') {
                    if v.0.trim() == "Data Source" {
                        data_source = Some(v.1.trim().to_string());
                    }
                    if v.0.trim() == "User Id" {
                        user_id = Some(v.1.trim().to_string());
                    }
                }
            }

            if data_source.is_none() || user_id.is_none() {
                return Err(format!("{:?}: Provider not found.", file));
            }
            Ok(Some(Connection {
                path: file.to_string_lossy().to_string(),
                data_source: data_source.unwrap(),
                user_id: user_id.unwrap(),
                provider,
            }))
        }
        Err(e) => Err(format!("{:?}: {}", file, e)),
    }
}

/// Extract Connection from element string in Classic ASP format.
fn extract_from_asp_classic(element: &str, file: &Path) -> Result<Option<Connection>, String> {
    if !element.contains("Provider") {
        return Ok(None);
    }

    let mut data_source = None;
    let mut user_id = None;
    let mut provider = None;
    for parts in element.split(';') {
        if let Some(v) = parts
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim()
            .split_once('=')
        {
            if v.0.trim() == "Data Source" {
                data_source = Some(v.1.trim().to_string());
            }
            if v.0.trim() == "User Id" {
                user_id = Some(v.1.trim().to_string());
            }
            if v.0.trim() == "Provider" {
                provider = Some(v.1.trim().to_string());
            }
        }
    }
    if data_source.is_none() || user_id.is_none() || provider.is_none() {
        return Err(format!(
            "{:?}: Provider, Data Source, or User Id not found",
            file
        ));
    }
    Ok(Some(Connection {
        path: file.to_string_lossy().to_string(),
        data_source: data_source.unwrap(),
        user_id: user_id.unwrap(),
        provider: provider.unwrap(),
    }))
}

fn main() -> std::io::Result<()> {
    // crawl files and extract db connections

    // use dir provided by command line argument or default to cwd
    let dir = if let Some(v) = env::args().nth(1) {
        v
    } else {
        ".".to_string()
    };
    let dir = Path::new(&dir);
    if !dir.is_dir() {
        println!(
            "Cannot find directory {:?}. Please provide a valid directory.",
            dir
        );
        return Ok(());
    }

    if let Ok(v) = get_files(dir.to_path_buf(), vec![]) {
        if v.is_empty() {
            println!("Could not find any matching files.");
            return Ok(());
        }

        let (connections, errors) = extract_connections_from_files(v);

        // write to connections to CSV file
        if !connections.is_empty() {
            let mut wtr = Writer::from_path("output.csv")?;
            wtr.write_record(["path", "data source", "user id", "provider"])?;
            for c in connections {
                wtr.write_record(&[c.path, c.data_source, c.user_id, c.provider])?;
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_from_asp_net_succeeds() {
        let element_str = r#"<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerName="System.OracleClient"/>""#;
        let file = Path::new("some_path");
        let c = extract_from_asp_net(element_str, file);
        assert!(c.is_ok());
        assert_eq!(c.clone().unwrap().unwrap().data_source, "db2".to_string());
        assert_eq!(c.unwrap().unwrap().user_id, "dvrpc".to_string())
    }
    #[test]
    fn extract_from_asp_classic_succeeds() {
        let element_str = "Data Source=db2; User Id=dvrpc; Provider=System.OracleClient";
        let file = Path::new("some_path");
        let c = extract_from_asp_classic(element_str, file);
        assert!(c.is_ok());
        assert_eq!(c.clone().unwrap().unwrap().data_source, "db2".to_string());
        assert_eq!(c.unwrap().unwrap().user_id, "dvrpc".to_string())
    }

    // test traversal of files
    #[test]
    fn get_files_finds_test_files() {
        let files = get_files(Path::new("test_files/").to_path_buf(), vec![]);
        assert!(files.is_ok())
    }
    #[test]
    fn get_files_skips_proper_files() {
        let files = get_files(Path::new("test_files/").to_path_buf(), vec![]).unwrap();
        // "unmatched.extension" should be absent from list of files
        // but "test.config" should exist
        assert!(!files
            .iter()
            .any(|x| x.as_path() == Path::new("test_files/unmatched.extension")));
        assert!(files
            .iter()
            .any(|x| x.as_path() == Path::new("test_files/test.config")))
    }
    // test_files directory is set up to produce specific successes and errors
    // from multiple files
    #[test]
    fn count_of_connections_and_errors_from_test_files_is_correct() {
        let dir = Path::new("test_files");
        if let Ok(v) = get_files(dir.to_path_buf(), vec![]) {
            let (connections, errors) = extract_connections_from_files(v);

            assert!(connections.len() == 13 && errors.len() == 18);
        }
    }
}

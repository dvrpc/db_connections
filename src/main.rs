use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use csv::Writer;
use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use simplelog::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AspNet {
    #[serde(alias = "connectionString")]
    #[serde(alias = "Connectionstring")]
    #[serde(alias = "ConnectionString")]
    #[serde(alias = "connectionstring")]
    connection_string: String,
    #[serde(alias = "Provider")]
    #[serde(alias = "provider")]
    #[serde(alias = "providerName")]
    #[serde(alias = "Providername")]
    #[serde(alias = "ProviderName")]
    #[serde(alias = "providername")]
    provider: Option<String>,
}

#[derive(Debug, Clone)]
struct Connection {
    path: String,
    data_source: String,
    user_id: String,
    provider: String,
}

#[derive(Debug, Clone)]
struct DbError {
    path: String,
    message: String,
}

/// Get files matching extensions we want.
fn get_files(dir: PathBuf, mut files: Vec<PathBuf>) -> std::io::Result<Vec<PathBuf>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files = get_files(entry.path(), files)?
        } else if let Some(v) = entry.path().extension() {
            if let Some(v) = v.to_str() {
                if ["config", "aspx", "asp", "json"].contains(&v) {
                    files.push(entry.path().clone());
                }
            }
        }
    }
    Ok(files)
}

/// Iterate through files and extract Connections.
fn extract_connections_from_files(files: Vec<PathBuf>) -> (Vec<Connection>, Vec<DbError>) {
    let mut connections = vec![];
    let mut errors = vec![];

    for file in files {
        let extension = file.extension().unwrap().to_str().unwrap();
        let content: String;

        if extension == "config" || extension == "aspx" || extension == "json" {
            // read file to string and then capture regular expression matches
            // (anything in angle brackets)
            match fs::read_to_string(file.clone()) {
                Ok(v) => content = v.to_string(),
                Err(e) => {
                    errors.push(DbError {
                        path: file.to_string_lossy().to_string(),
                        message: format!("Could not read file ({e})"),
                    });
                    continue;
                }
            }

            // format1
            let re = Regex::new(r#"(?s)<add .*?>"#).unwrap();
            let mut results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            // format2
            let re2 = Regex::new(r#"(?s)\(".*?"\)"#).unwrap();
            let mut results2 = re2
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            results.append(&mut results2);

            for result in results {
                // format2
                if result.starts_with('(') {
                    match extract_from_asp_net_format2(result, &file) {
                        Ok(None) => (),
                        Ok(Some(v)) => connections.push(v),
                        Err(e) => errors.push(DbError {
                            path: file.to_string_lossy().to_string(),
                            message: e.to_string(),
                        }),
                    }
                // format1
                } else {
                    match extract_from_asp_net_format1(result, &file) {
                        Ok(None) => (),
                        Ok(Some(v)) => connections.push(v),
                        Err(e) => errors.push(DbError {
                            path: file.to_string_lossy().to_string(),
                            message: e.to_string(),
                        }),
                    }
                }
            }
        } else if extension == "asp" {
            // read file to string and then capture regular expression matches
            // (anything in double quotes)
            match fs::read_to_string(file.clone()) {
                Ok(v) => content = v.to_string(),
                Err(e) => {
                    errors.push(DbError {
                        path: file.to_string_lossy().to_string(),
                        message: format!("Could not read file ({e})"),
                    });
                    continue;
                }
            }
            let re = Regex::new(r#"(?s)".*?""#).unwrap();
            let results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            for result in results {
                match extract_from_asp_classic(result, &file) {
                    Ok(None) => (),
                    Ok(Some(v)) => connections.push(v),
                    Err(e) => errors.push(DbError {
                        path: file.to_string_lossy().to_string(),
                        message: e.to_string(),
                    }),
                }
            }
        }
    }
    (connections, errors)
}

/// Extract Connection from element string in ASP.NET format1 (<add * >)
fn extract_from_asp_net_format1(element: &str, file: &Path) -> Result<Option<Connection>, String> {
    if !element.to_lowercase().contains("connection") {
        return Ok(None);
    }

    match serde_xml_rs::from_str::<AspNet>(element) {
        Ok(mut v) => {
            let mut data_source = None;
            let mut user_id = None;

            for pair in v.connection_string.split(';') {
                if let Some(w) = pair.trim().split_once('=') {
                    if w.0.to_lowercase().trim() == "data source" {
                        data_source = Some(w.1.trim().to_string());
                    }
                    if w.0.to_lowercase().trim() == "user id" {
                        user_id = Some(w.1.trim().to_string());
                    }
                    // in some cases, provider can be within connectionString
                    // Use that (and overwrite any previous value from
                    // a separate attribute)
                    if w.0.to_lowercase().trim() == "provider"
                        || w.0.to_lowercase().trim() == "providername"
                    {
                        v.provider = Some(w.1.trim().to_string());
                    }
                }
            }

            if data_source.is_none() || user_id.is_none() || v.provider.is_none() {
                return Err("Provider, Data Source, or User Id not found.".to_string());
            }
            Ok(Some(Connection {
                path: file.to_string_lossy().to_string(),
                data_source: data_source.unwrap(),
                user_id: user_id.unwrap(),
                provider: v.provider.unwrap(),
            }))
        }
        Err(e) => Err(e.to_string()),
    }
}
/// Extract Connection from element string in ASP.NET format1 ( ("*") )
fn extract_from_asp_net_format2(
    mut element: &str,
    file: &Path,
) -> Result<Option<Connection>, String> {
    // trim to just the part within `("")`
    element = element.trim_start_matches("(\"");
    element = element.trim_end_matches("\")");

    let mut data_source = None;
    let mut user_id = None;
    let mut provider = None;

    for pair in element.split(';') {
        if let Some(w) = pair.trim().split_once('=') {
            if w.0.to_lowercase().trim() == "data source" {
                data_source = Some(w.1.trim().to_string());
            }
            if w.0.to_lowercase().trim() == "user id" {
                user_id = Some(w.1.trim().to_string());
            }
            if w.0.to_lowercase().trim() == "provider"
                || w.0.to_lowercase().trim() == "providername"
            {
                provider = Some(w.1.trim().to_string());
            }
        }
    }

    if data_source.is_none() || user_id.is_none() || provider.is_none() {
        return Err("Provider, Data Source, or User Id not found.".to_string());
    }

    Ok(Some(Connection {
        path: file.to_string_lossy().to_string(),
        data_source: data_source.unwrap(),
        user_id: user_id.unwrap(),
        provider: provider.unwrap(),
    }))
}

/// Extract Connection from element string in Classic ASP format.
fn extract_from_asp_classic(element: &str, file: &Path) -> Result<Option<Connection>, String> {
    if !element.to_lowercase().contains("provider") {
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
            if v.0.to_lowercase().trim() == "data source" {
                data_source = Some(v.1.trim().to_string());
            }
            if v.0.to_lowercase().trim() == "user id" {
                user_id = Some(v.1.trim().to_string());
            }
            if v.0.to_lowercase().trim() == "provider"
                || v.0.to_lowercase().trim() == "providername"
            {
                provider = Some(v.1.trim().to_string());
            }
        }
    }
    if data_source.is_none() || user_id.is_none() || provider.is_none() {
        return Err("Provider, Data Source, or User Id not found".to_string());
    }
    Ok(Some(Connection {
        path: file.to_string_lossy().to_string(),
        data_source: data_source.unwrap(),
        user_id: user_id.unwrap(),
        provider: provider.unwrap(),
    }))
}

/// Crawl files and write connections and errors to CSV files.
fn main() -> std::io::Result<()> {
    // Log a few messages related to running this program itself,
    // not the processing of files (those go into the generated CSV files).
    // Build the Config to be used for logging
    let config = ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] \
                [offset_hour sign:mandatory][offset_minute]"
        ))
        .build();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            config,
            OpenOptions::new()
                .append(true)
                .create(true)
                .open("db_connections.log")?,
        ),
    ])
    .unwrap();

    info!("Program started.");

    // use dirs provided by command line arguments or default to cwd
    let mut dirs = env::args().skip(1).collect::<Vec<_>>();

    if dirs.is_empty() {
        dirs = vec![".".to_string()]
    }
    info!("Running on directories: {:?}", dirs);

    let mut files = vec![];

    for dir in dirs {
        let dir = Path::new(&dir);

        if !dir.is_dir() {
            error!("Could not find directory {:?} - skipping.", dir);
        } else if let Ok(v) = get_files(dir.to_path_buf(), vec![]) {
            if v.is_empty() {
                info!("Could not find any matching files in {:?}.", dir);
                continue;
            } else {
                files.extend(v)
            }
        }
    }

    let (connections, errors) = extract_connections_from_files(files);

    // write connections to file
    let mut wtr = Writer::from_path("connections.csv")?;
    wtr.write_record(["path", "data source", "user id", "provider"])?;
    for c in connections {
        wtr.write_record(&[c.path, c.data_source, c.user_id, c.provider])?;
    }
    wtr.flush()?;

    // write errors to file
    let mut wtr = Writer::from_path("errors.csv")?;
    wtr.write_record(["path", "error"])?;
    for e in errors {
        wtr.write_record(&[e.path, e.message])?;
    }
    wtr.flush()?;

    info!("Finished.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const PROVIDER_SPELLINGS: [&str; 6] = [
        "provider",
        "Provider",
        "providerName",
        "Providername",
        "ProviderName",
        "providername",
    ];

    #[test]
    fn extract_from_asp_net_format1_succeeds() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str = &format!(
                r#"<add name="nets" 
                connectionString="Data Source=db2; User Id=dvrpc;" 
                {provider}="System.OracleClient"/>""#
            );
            let c = extract_from_asp_net_format1(element_str, file);
            assert!(c.is_ok());
            assert_eq!(c.clone().unwrap().unwrap().data_source, "db2".to_string());
            assert_eq!(c.clone().unwrap().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(
                c.unwrap().unwrap().provider,
                "System.OracleClient".to_string()
            )
        }
    }
    #[test]
    fn extract_from_asp_net_format2_succeeds() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str = &format!(
                r#"("{provider}=OraOLEDB.Oracle;
                Data Source=dvrpcdb2;User ID=dvrpc;Password=something;")"#
            );
            let c = extract_from_asp_net_format2(element_str, file);
            assert!(c.is_ok());
            assert_eq!(
                c.clone().unwrap().unwrap().data_source,
                "dvrpcdb2".to_string()
            );
            assert_eq!(c.clone().unwrap().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(c.unwrap().unwrap().provider, "OraOLEDB.Oracle".to_string())
        }
    }
    #[test]
    fn extract_from_asp_classic_succeeds() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str =
                &format!("Data Source=db2; User Id=dvrpc; {provider}=System.OracleClient",);
            let c = extract_from_asp_classic(element_str, file);
            assert!(c.is_ok());
            assert_eq!(c.clone().unwrap().unwrap().data_source, "db2".to_string());
            assert_eq!(c.clone().unwrap().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(
                c.unwrap().unwrap().provider,
                "System.OracleClient".to_string()
            )
        }
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
            assert_eq!(connections.len(), 18);
            assert_eq!(errors.len(), 13)
        }
    }
}

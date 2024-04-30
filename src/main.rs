use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use csv::Writer;
use log::{error, info};
use regex::Regex;
use simplelog::*;

#[derive(Debug, Clone)]
struct Connection {
    path: String,
    data_source: String,
    user_id: String,
    provider: String,
}

#[derive(Debug, Clone)]
struct DbConnectionError {
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
fn extract_connections_from_files(
    files: Vec<PathBuf>,
) -> (Vec<Connection>, Vec<DbConnectionError>) {
    let mut connections = vec![];
    let mut errors = vec![];

    for file in files {
        let extension = file.extension().unwrap().to_str().unwrap();
        let content: String;

        if extension == "config" || extension == "aspx" {
            // read file to string and then capture regular expression matches
            // (anything in angle brackets)
            match fs::read_to_string(file.clone()) {
                Ok(v) => content = v.to_string(),
                Err(e) => {
                    errors.push(DbConnectionError {
                        path: file.to_string_lossy().to_string(),
                        message: format!("Could not read file ({e})"),
                    });
                    continue;
                }
            }

            // ASP.NET format
            let re = Regex::new(r#"(?s)<add .*?>"#).unwrap();
            let mut results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            // ASP Classic format
            let re2 = Regex::new(r#"(?s)\(".*?"\)"#).unwrap();
            let mut results2 = re2
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            results.append(&mut results2);

            for result in results {
                // ASP Classic format
                if result.starts_with('(') {
                    match extract_from_asp_classic(result, &file) {
                        None => (),
                        Some(v) => connections.push(v),
                    }
                // ASP.NET format
                } else {
                    match extract_from_asp_net(result, &file) {
                        None => (),
                        Some(v) => connections.push(v),
                    }
                }
            }
        } else if extension == "json" {
            // read file to string and then capture regular expression matches
            // (anything follow "ConnectionStrings")
            match fs::read_to_string(file.clone()) {
                Ok(v) => content = v.to_string(),
                Err(e) => {
                    errors.push(DbConnectionError {
                        path: file.to_string_lossy().to_string(),
                        message: format!("Could not read file ({e})"),
                    });
                    continue;
                }
            }

            let re = Regex::new(r#"(?s)ConnectionStrings.*?}"#).unwrap();
            let results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            for result in results {
                let mut json_results = extract_from_json(result, &file);
                connections.append(&mut json_results);
            }
        } else if extension == "asp" {
            // read file to string and then capture regular expression matches
            // (anything in double quotes)
            match fs::read_to_string(file.clone()) {
                Ok(v) => content = v.to_string(),
                Err(e) => {
                    errors.push(DbConnectionError {
                        path: file.to_string_lossy().to_string(),
                        message: format!("Could not read file ({e})"),
                    });
                    continue;
                }
            };
            let re = Regex::new(r#"(?s)".*?""#).unwrap();
            let results = re
                .captures_iter(&content)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect::<Vec<_>>();

            for result in results {
                match extract_from_asp_classic(result, &file) {
                    None => (),
                    Some(v) => connections.push(v),
                }
            }
        }
    }
    (connections, errors)
}

/// Extract Connection from element string in ASP.NET (<add * >)
fn extract_from_asp_net(element: &str, file: &Path) -> Option<Connection> {
    if !element.to_lowercase().contains("connection") {
        return None;
    }

    let mut data_source = String::new();
    let mut user_id = String::new();
    let mut provider = String::new();

    // First, try to get all information from the ConnectionString attribute.
    let re = Regex::new(r#"(?s)[C|c]onnection[S|s]tring=".*?""#).unwrap();
    let connection_strings = re
        .captures_iter(element)
        .map(|cap| cap.get(0).unwrap().as_str())
        .collect::<Vec<_>>()
        .join("")
        .trim_matches('"')
        .to_string();

    // remove initial part, "connectionString="
    let connection_strings = connection_strings.split_once('=').unwrap().1;

    for pair in connection_strings.split(';') {
        if let Some(w) = pair.trim().trim_matches('"').split_once('=') {
            if w.0.to_lowercase().trim().contains("data source") {
                data_source = w.1.trim().to_string();
            }
            if w.0.to_lowercase().trim().contains("user id") {
                user_id = w.1.trim().to_string();
            }
            if w.0.to_lowercase().trim().contains("provider") {
                provider = w.1.trim().to_string();
            }
        }
    }

    // If provider wasn't found in connection string, check for separate attribute.
    if provider.is_empty() {
        let re = Regex::new(r#"(?s)[P|p]rovider.*?=".*?""#).unwrap();
        let provider_caps = re
            .captures_iter(element)
            .map(|cap| cap.get(0).unwrap().as_str())
            .collect::<Vec<_>>()
            .join("");

        if let Some(v) = provider_caps.trim().split_once('=') {
            provider = v.1.trim().trim_matches('"').to_string();
        }
    }

    if !data_source.is_empty() || !user_id.is_empty() || !provider.is_empty() {
        Some(Connection {
            path: file.to_string_lossy().to_string(),
            data_source,
            user_id,
            provider,
        })
    } else {
        None
    }
}

/// Extract Connection from element string in ASP.NET format2 or ASP Classic ( ("*") )
fn extract_from_asp_classic(mut element: &str, file: &Path) -> Option<Connection> {
    // trim to just the part within `("")`
    element = element.trim_start_matches("(\"");
    element = element.trim_end_matches("\")");

    let mut data_source = String::new();
    let mut user_id = String::new();
    let mut provider = String::new();

    for pair in element.split(';') {
        if let Some(w) = pair.trim().trim_matches('"').split_once('=') {
            if w.0.to_lowercase().trim().contains("data source") {
                data_source = w.1.trim().to_string();
            }
            if w.0.to_lowercase().trim().contains("user id") {
                user_id = w.1.trim().to_string();
            }
            if w.0.to_lowercase().trim().contains("provider") {
                provider = w.1.trim().to_string();
            }
        }
    }

    if !data_source.is_empty() || !user_id.is_empty() || !provider.is_empty() {
        Some(Connection {
            path: file.to_string_lossy().to_string(),
            data_source,
            user_id,
            provider,
        })
    } else {
        None
    }
}

/// Extract Connection from element string in json settings file.
///
/// There can be multiple connection strings, so this one returns a Vec.
fn extract_from_json(element: &str, file: &Path) -> Vec<Connection> {
    let mut results = vec![];
    for line in element.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut data_source = String::new();
        let mut user_id = String::new();
        let mut provider = String::new();

        for parts in line.split(';') {
            if let Some(v) = parts
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim()
                .split_once('=')
            {
                if v.0.to_lowercase().trim().contains("data source") {
                    data_source = v.1.trim().to_string();
                }
                if v.0.to_lowercase().trim().contains("user id") {
                    user_id = v.1.trim().to_string();
                }
                if v.0.to_lowercase().trim().contains("provider") {
                    provider = v.1.trim().to_string();
                }
            }
        }
        // Only push Connection to Vec if any field contains data (all aren't empty).
        if !data_source.is_empty() || !user_id.is_empty() || !provider.is_empty() {
            results.push(Connection {
                path: file.to_string_lossy().to_string(),
                data_source,
                user_id,
                provider,
            })
        }
    }
    results
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
    fn extract_from_asp_net_succeeds() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str = &format!(
                r#"<add name="nets" 
                connectionString="Data Source=db2; User Id=dvrpc;" 
                {provider}="System.OracleClient"/>""#
            );
            let c = extract_from_asp_net(element_str, file);
            assert!(c.is_some());
            assert_eq!(c.clone().unwrap().data_source, "db2".to_string());
            assert_eq!(c.clone().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(c.unwrap().provider, "System.OracleClient".to_string())
        }
    }

    #[test]
    fn extract_from_asp_classic_succeeds_1() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str = &format!(
                r#"("{provider}=OraOLEDB.Oracle;
                Data Source=dvrpcdb2;User ID=dvrpc;Password=something;")"#
            );
            let c = extract_from_asp_classic(element_str, file);
            assert!(c.is_some());
            assert_eq!(c.clone().unwrap().data_source, "dvrpcdb2".to_string());
            assert_eq!(c.clone().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(c.unwrap().provider, "OraOLEDB.Oracle".to_string())
        }
    }

    #[test]
    fn extract_from_asp_classic_succeeds_2() {
        let file = Path::new("some_path");
        for provider in PROVIDER_SPELLINGS {
            let element_str =
                &format!("Data Source=db2; User Id=dvrpc; {provider}=System.OracleClient",);
            let c = extract_from_asp_classic(element_str, file);
            assert!(c.is_some());
            assert_eq!(c.clone().unwrap().data_source, "db2".to_string());
            assert_eq!(c.clone().unwrap().user_id, "dvrpc".to_string());
            assert_eq!(c.unwrap().provider, "System.OracleClient".to_string())
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
            assert_eq!(connections.len(), 30);
            assert_eq!(errors.len(), 0)
        }
    }
}

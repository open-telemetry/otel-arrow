// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::print_stderr, clippy::print_stdout)]

//! Minimal Oracle OCI connectivity and query smoke test.

use oracle::Connection;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_QUERY: &str = "SELECT SYSDATE AS CURRENT_TIME FROM DUAL";
const DEFAULT_MAX_ROWS: usize = 10;
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

struct Config {
    connect_string: String,
    username_file: PathBuf,
    password_file: PathBuf,
    query: String,
    max_rows: usize,
}

impl Config {
    fn from_args() -> Result<Self, String> {
        let mut args = std::env::args().skip(1);
        let connect_string = args.next().ok_or_else(usage)?;
        let username_file = args.next().map(PathBuf::from).ok_or_else(usage)?;
        let password_file = args.next().map(PathBuf::from).ok_or_else(usage)?;
        let query = args.next().unwrap_or_else(|| DEFAULT_QUERY.to_owned());
        let max_rows = args
            .next()
            .map(|value| parse_max_rows(&value))
            .transpose()?
            .unwrap_or(DEFAULT_MAX_ROWS);
        if args.next().is_some() {
            return Err(usage());
        }
        Ok(Self {
            connect_string,
            username_file,
            password_file,
            query,
            max_rows,
        })
    }
}

fn main() {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!("{}", usage());
        return;
    }
    if let Err(error) = run() {
        eprintln!("Oracle OCI smoke test failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::from_args()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let username = read_secret(&config.username_file)?;
    let password = read_secret(&config.password_file)?;

    println!("Connecting to Oracle at {}...", config.connect_string);
    let connection = Connection::connect(&username, &password, &config.connect_string)?;
    connection.set_call_timeout(Some(CALL_TIMEOUT))?;
    connection.ping()?;
    println!("Connection succeeded.");

    let row_count = read_rows(&connection, &config.query, config.max_rows)?;
    println!("Query succeeded. Read {row_count} row(s).");
    connection.close()?;
    Ok(())
}

fn read_rows(
    connection: &Connection,
    query: &str,
    max_rows: usize,
) -> Result<usize, oracle::Error> {
    let fetch_size = u32::try_from(max_rows).unwrap_or(u32::MAX);
    let mut statement = connection
        .statement(query)
        .fetch_array_size(fetch_size)
        .build()?;
    let mut rows = statement.query(&[])?;
    let columns = rows
        .column_info()
        .iter()
        .map(|column| column.name().to_owned())
        .collect::<Vec<_>>();
    let mut row_count = 0;

    for row in rows.by_ref().take(max_rows) {
        let row = row?;
        row_count += 1;
        print!("row {row_count}: ");
        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                print!(", ");
            }
            let value = row
                .get::<_, Option<String>>(index)?
                .unwrap_or_else(|| "NULL".to_owned());
            print!("{column}={value}");
        }
        println!();
    }
    Ok(row_count)
}

fn read_secret(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut value = std::fs::read_to_string(path)?;
    if value.ends_with("\r\n") {
        value.truncate(value.len() - 2);
    } else if value.ends_with('\n') {
        _ = value.pop();
    }
    if value.is_empty() {
        return Err(format!("credential file {} is empty", path.display()).into());
    }
    if value.contains('\0') {
        return Err(format!("credential file {} contains a NUL byte", path.display()).into());
    }
    Ok(value)
}

fn parse_max_rows(value: &str) -> Result<usize, String> {
    let max_rows = value
        .parse::<usize>()
        .map_err(|_| "max_rows must be a positive integer".to_owned())?;
    if max_rows == 0 {
        return Err("max_rows must be a positive integer".to_owned());
    }
    Ok(max_rows)
}

fn usage() -> String {
    format!(
        "usage: otap-df-oracle-oci-smoke <connect-string> <username-file> \
         <password-file> [query] [max-rows]\n\
         default query: {DEFAULT_QUERY:?}\n\
         Oracle Instant Client must be installed and available to the process"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the optional row limit is a valid positive integer.
    /// Guarantees: the smoke test accepts the bound used to stop row iteration.
    #[test]
    fn parses_positive_max_rows() {
        assert_eq!(parse_max_rows("25").unwrap(), 25);
    }

    /// Scenario: the optional row limit is zero or malformed.
    /// Guarantees: invalid limits fail before any Oracle connection is attempted.
    #[test]
    fn rejects_invalid_max_rows() {
        assert!(parse_max_rows("0").is_err());
        assert!(parse_max_rows("many").is_err());
    }

    /// Scenario: a mounted credential contains the usual trailing newline.
    /// Guarantees: only the line ending is removed before authentication.
    #[test]
    fn reads_secret_file() {
        let path =
            std::env::temp_dir().join(format!("oracle-oci-smoke-secret-{}", std::process::id()));
        std::fs::write(&path, "reader\r\n").unwrap();
        let result = read_secret(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result, "reader");
    }
}

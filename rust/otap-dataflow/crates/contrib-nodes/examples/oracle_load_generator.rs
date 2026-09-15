// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deterministic load generator for the Oracle receiver's watermark table.

#![allow(clippy::print_stdout)]

use oracle::Connection;
use std::error::Error;

const TABLE: &str = "OTAP_ORACLE_EVENTS";
const CREATE_TABLE: &str = "CREATE TABLE OTAP_ORACLE_EVENTS (
    EVENT_TS TIMESTAMP(9) NOT NULL,
    EVENT_ID NUMBER(19) NOT NULL PRIMARY KEY,
    PAYLOAD VARCHAR2(200) NOT NULL
)";
const MERGE_ROW: &str = "MERGE INTO OTAP_ORACLE_EVENTS target
USING (
    SELECT
        CAST(:1 AS NUMBER(19)) AS EVENT_ID,
        TO_TIMESTAMP('2026-01-01 00:00:00', 'YYYY-MM-DD HH24:MI:SS')
            + NUMTODSINTERVAL(:2, 'SECOND') AS EVENT_TS,
        CAST(:3 AS VARCHAR2(200)) AS PAYLOAD
    FROM DUAL
) source
ON (target.EVENT_ID = source.EVENT_ID)
WHEN NOT MATCHED THEN INSERT (EVENT_TS, EVENT_ID, PAYLOAD)
VALUES (source.EVENT_TS, source.EVENT_ID, source.PAYLOAD)";

struct Options {
    rows: i64,
    collision_size: i64,
    reset: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let username = required_env("ORACLE_USERNAME")?;
    let password = required_env("ORACLE_PWD")?;
    let connect_string = required_env("ORACLE_CONNECT_STRING")?;
    let connection = Connection::connect(username, password, connect_string)?;

    if options.reset {
        match connection.execute(&format!("DROP TABLE {TABLE} PURGE"), &[]) {
            Ok(_) => {}
            Err(error)
                if error
                    .db_error()
                    .is_some_and(|db_error| db_error.code() == 942) => {}
            Err(error) => return Err(error.into()),
        }
    }
    match connection.execute(CREATE_TABLE, &[]) {
        Ok(_) => {}
        Err(error)
            if error
                .db_error()
                .is_some_and(|db_error| db_error.code() == 955) => {}
        Err(error) => return Err(error.into()),
    }

    for event_id in 1..=options.rows {
        let timestamp_offset = (event_id - 1) / options.collision_size;
        let payload = format!("event-{event_id:012}");
        let _ = connection.execute(MERGE_ROW, &[&event_id, &timestamp_offset, &payload])?;
    }
    connection.commit()?;

    println!(
        "Prepared {TABLE} with {} deterministic rows and collision groups of {}",
        options.rows, options.collision_size
    );
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut rows = 1_000i64;
    let mut collision_size = 10i64;
    let mut reset = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--rows" => {
                rows = arguments.next().ok_or("--rows requires a value")?.parse()?;
            }
            "--collision-size" => {
                collision_size = arguments
                    .next()
                    .ok_or("--collision-size requires a value")?
                    .parse()?;
            }
            "--reset" => reset = true,
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    if rows <= 0 {
        return Err("--rows must be greater than zero".into());
    }
    if collision_size <= 0 {
        return Err("--collision-size must be greater than zero".into());
    }
    Ok(Options {
        rows,
        collision_size,
        reset,
    })
}

fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|error| format!("environment variable {name} is required: {error}").into())
}

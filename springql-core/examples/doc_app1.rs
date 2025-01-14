// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

//! Demo application in <https://springql.github.io/get-started/write-basic-apps/#app1-simple-arithmetic-conversion-over-a-stream>.
//!
//! Usage:
//!
//! ```bash
//! cargo run --example doc_app1
//! ```
//!
//! ```bash
//! echo '{"ts": "2022-01-01 13:00:00.000000000", "temperature": 5.3}' |nc localhost 54300
//! ```

use springql_core::{high_level_rs::SpringPipelineHL, low_level_rs::SpringConfig};

fn main() {
    const SOURCE_PORT: u16 = 54300;

    let pipeline = SpringPipelineHL::new(&SpringConfig::default()).unwrap();

    pipeline
        .command(
            "
            CREATE SOURCE STREAM source_temperature_celsius (
                ts TIMESTAMP NOT NULL ROWTIME,    
                temperature FLOAT NOT NULL
            );
            ",
        )
        .unwrap();

    pipeline
        .command(
            "
            CREATE SINK STREAM sink_temperature_fahrenheit (
                ts TIMESTAMP NOT NULL ROWTIME,    
                temperature FLOAT NOT NULL
            );
            ",
        )
        .unwrap();

    pipeline
        .command(
            "
            CREATE PUMP c_to_f AS
                INSERT INTO sink_temperature_fahrenheit (ts, temperature)
                SELECT STREAM
                    source_temperature_celsius.ts,
                    32.0 + source_temperature_celsius.temperature * 1.8
                FROM source_temperature_celsius;
            ",
        )
        .unwrap();

    pipeline
        .command(
            "
            CREATE SINK WRITER queue_temperature_fahrenheit FOR sink_temperature_fahrenheit
            TYPE IN_MEMORY_QUEUE OPTIONS (
                NAME 'q'
            );
            ",
        )
        .unwrap();

    pipeline
        .command(format!(
            "
            CREATE SOURCE READER tcp_temperature_celsius FOR source_temperature_celsius
            TYPE NET_SERVER OPTIONS (
                PROTOCOL 'TCP',
                PORT '{}'
            );
            ",
            SOURCE_PORT
        ))
        .unwrap();

    eprintln!("waiting JSON records in tcp/{} port...", SOURCE_PORT);

    while let Ok(row) = pipeline.pop("q") {
        let ts: String = row.get_not_null_by_index(0).unwrap();
        let temperature_fahrenheit: f32 = row.get_not_null_by_index(1).unwrap();
        eprintln!("{}\t{}", ts, temperature_fahrenheit);
    }
}

use std::collections::HashMap;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use chrono::{DateTime, Utc};

use snapshot_cache::snapshot_cache_core::collections::UpdatingMap;
use snapshot_cache::snapshot_cache_core::metrics::Metrics;
use snapshot_cache::snapshot_cache_core::util::{Error, Result};

use snapshot_cache::snapshot_cache_core::util::{OnFailure, OnUpdate};
use snapshot_cache::Fallback;
use snapshot_cache::LocalFileConfigSource;
use snapshot_cache::RawLineMapProcessor;
use snapshot_cache::SnapshotCache;

fn main() {
    let source = LocalFileConfigSource::new("./my.config");
    let processor = RawLineMapProcessor::new(parse_line);

    let cache = SnapshotCache::<UpdatingMap<u128, String, i32>>::map_builder()
        // These are required.
        .with_source(source)
        .with_processor(processor)
        .with_fetch_interval(Duration::from_secs(2))
        // These are optional
        .with_name("my-cache")
        .with_fallback(Fallback::with_value(HashMap::new()))
        .with_update_callback(OnUpdate::with_fn(|_, v, _| {
            println!("Updated to version {}", v.unwrap_or(0))
        }))
        .with_failure_callback(OnFailure::with_fn(|e, _| {
            println!("Failed with error: {}", e)
        }))
        .with_metrics(ExampleMetrics {})
        .build()
        .unwrap();

    // Collection instances are safe to hold on to, borrow, clone, or pass ownership of.
    let map = cache.cache();
    loop {
        println!("C={}", map.get(&String::from("C")).unwrap_or_default());
        sleep(Duration::from_secs(3));
    }
}

fn parse_line(raw: String) -> Result<Option<(String, i32)>> {
    if raw.trim().is_empty() || raw.starts_with('#') {
        return Ok(None);
    }

    if let Some((k, v)) = raw.split_once('=') {
        Ok(Some((String::from(k), i32::from_str(v)?)))
    } else {
        Err(Error::new(format!("Failed to parse '{}'", raw).as_str()))
    }
}

struct ExampleMetrics {}

impl Metrics<u128> for ExampleMetrics {
    fn update(&self, _new_version: &Option<u128>, fetch_time: Duration, process_time: Duration) {
        println!(
            "Update fetch took {}ms and process took {}ms",
            fetch_time.as_millis(),
            process_time.as_millis()
        );
    }

    fn last_successful_update(&self, ts: &DateTime<Utc>) {
        println!("Last successful update is now at {}", ts);
    }

    fn check_no_update(&self, check_time: &Duration) {
        println!("File hasn't changed. Check in {}ms", check_time.as_millis())
    }

    fn last_successful_check(&self, ts: &DateTime<Utc>) {
        println!("Last successful check is now at {}", ts);
    }

    fn fallback_invoked(&self) {
        println!("Fallback invoked!");
    }

    fn fetch_error(&self, err: &Error) {
        println!("Fetch failed with: '{}'", err)
    }

    fn process_error(&self, err: &Error) {
        println!("Process failed with: '{}'", err)
    }
}

use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use reqwest::blocking as reqwest;
use serde_json::Value;


// These correspond to what was in config.def.h.
const API_KEY: &str = ""; // Your API key for Open Exchange Rates. Get your own by signing up at https://openexchangerates.org/signup/free
const API_URL: &str = "https://openexchangerates.org/api/latest.json";

// Relative path under $HOME
const FILE_NAME: &str = ".cache/currency.db";

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments: currency FROM TO amount
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        eprintln!("currency -- Currency converter.");
        eprintln!("Usage:   currency FROM TO amount");
        eprintln!("Example: currency USD EUR 123.45");
        return Ok(()); // exit cleanly, like the original code
    }

    let from = args[1].to_uppercase();
    let to = args[2].to_uppercase();
    let amount: f64 = args[3].parse().unwrap_or(0.0);

    // Build the path: $HOME + FILE_NAME
    let home_dir = env::var("HOME").expect("Could not find $HOME environment variable");
    let mut file_path = PathBuf::from(home_dir);
    file_path.push(FILE_NAME);

    // Decide if we need to refresh the cache
    let mut need_refresh = true;
    if let Ok(metadata) = fs::metadata(&file_path) {
        if let Ok(mtime) = metadata.modified() {
            // Compare modification time with current time
            let now = SystemTime::now();
            if let Ok(age) = now.duration_since(mtime) {
                if age < Duration::from_secs(3600) {
                    // If less than 1 hour old, do not refresh
                    need_refresh = false;
                }
            }
        }
    }

    // Refresh from remote API if needed
    if need_refresh {
        let url = format!("{}?app_id={}", API_URL, API_KEY);
        match refresh_rates(&url, &file_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Warning: unable to refresh currency rates ({}). Trying to use previous data.",
                    e
                );
            }
        }
    }

    // Read JSON from cache file
    let json_string = match fs::read_to_string(&file_path) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!(
                "Error: unable to read currency rates from {}. Verify the file exists and permissions.",
                file_path.display()
            );
            std::process::exit(1);
        }
    };

    // Parse the JSON, extract "rates"
    let v: Value = serde_json::from_str(&json_string)
        .map_err(|_| "Could not parse JSON from the currency file")?;
    let rates = match &v["rates"] {
        Value::Object(_) => &v["rates"],
        _ => {
            eprintln!("Error: No 'rates' field found in the JSON data.");
            std::process::exit(1);
        }
    };

    // Look up the FROM and TO rates
    let rate_from = match rates.get(&from) {
        Some(val) => val.as_f64().unwrap_or(0.0),
        None => {
            eprintln!("Error: '{}' is not recognized as a currency.", from);
            std::process::exit(1);
        }
    };

    let rate_to = match rates.get(&to) {
        Some(val) => val.as_f64().unwrap_or(0.0),
        None => {
            eprintln!("Error: '{}' is not recognized as a currency.", to);
            std::process::exit(1);
        }
    };

    // Convert: (amount / rate_from) * rate_to
    let converted = (amount / rate_from) * rate_to;

    // Print result
    println!("{from} {:.4} = {to} {:.4}", amount, converted);

    Ok(())
}

/// Attempts to refresh the local cache file by fetching currency data from the given URL.
fn refresh_rates(url: &str, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let response = reqwest::get(url)?;
    if !response.status().is_success() {
        return Err(format!("HTTP request failed with status: {}", response.status()).into());
    }

    let content = response.bytes()?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)?;

    file.write_all(&content)?;

    Ok(())
}


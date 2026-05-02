// use once_cell::sync::Lazy;
// use std::sync::{Arc, Mutex};
use std::io::{self, Write};
mod app;
mod downloader;
mod media;
mod network;
mod utils;
use utils::clog;
mod config;
use config::{DN, TLD};

// static COUNTER: Lazy<Arc<Mutex<i32>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("[+] Starting MAD CLI");
    clog(utils::LogLvl::Info, "Starting MAD CLI");

    print!("Enter the url: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read the url");

    // println!("Target URL: {}", &input);
    clog(
        utils::LogLvl::Info,
        format!("Target URL: {}", &input.trim()).as_str(),
    );

    let input = input.trim();

    let input = if input.starts_with("https://") && input.contains(TLD) {
        input.to_string()
    } else {
        format!("https://{}.{}/{}", DN, TLD, input)
    };

    if let Err(e) = app::download(&input).await {
        eprintln!("Error during download: {}", e);
    }

    Ok(())
}

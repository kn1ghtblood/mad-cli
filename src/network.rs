use crate::config::FILES_CDN;
use regex::Regex;
use ureq;
// use std::{thread, time::Duration};

pub async fn get_uuid(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = ureq::get(url).call()?.into_string()?;
    let pattern = format!(
        r"https:\\/\\/{}\\/([^\\/]+)\\/seek\\/_0\.jpg",
        FILES_CDN.replace(".", r"\.")
    );
    let re = Regex::new(&pattern)?;
    re.captures(&res)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| "Failed to match uuid.".into())
}

pub fn fetch_playlist(url: &str) -> Result<String, String> {
    // 1. Setup an agent with a timeout (prevents hanging)
    // let agent: Agent = AgentBuilder::new()
    //     .timeout(Duration::from_secs(10))
    //     .build();

    // 2. Make the call
    match ureq::get(url).call() {
        Ok(response) => {
            // 3. Handle successful response but potential UTF-8 errors
            response
                .into_string()
                .map_err(|e| format!("Failed to read response body: {}", e))
        }
        Err(ureq::Error::Status(code, _)) => {
            // 4. Handle specific HTTP errors (404, 500, etc.)
            Err(format!("Server returned error code: {}", code))
        }
        Err(e) => {
            // 5. Handle transport errors (Connection refused, DNS, etc.)
            Err(format!("Network error: {}", e))
        }
    }
}

// pub fn request_with_retry(url: &str) -> Option<Vec<u8>> {
//     let max_retries = 5;
//     let delay = Duration::from_secs(2);

//     for _ in 0..max_retries {
//         match ureq::get(url).call() {
//             Ok(res) if res.status() == 200 => {
//                 let mut bytes = Vec::new();
//                 if res.into_reader().read_to_end(&mut bytes).is_ok() {
//                     return Some(bytes);
//                 }
//             }
//             _ => thread::sleep(delay),
//         }
//     }
//     eprintln!("Request timed out! Check the URL: {}", url);
//     None
// }

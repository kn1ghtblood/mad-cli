use crate::utils::{clog, LogLvl};
use bytes::Bytes;
use crate::config::{SAVE_PATH, VIDEO_CDN, ORIGIN, REFERER};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub async fn download_and_concat_hls(
    uuid: &str,
    resolution: &str,
    movie_name: &str,
    total_frames: usize,
    concurrency_limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Starting Concurrent Download & Stream Concatenation...");
    clog(
        LogLvl::Info,
        format!("Starting Concurrent Download & Stream Concatenation...").as_str(),
    );

    // 1. Setup Client with a strict timeout to prevent hanging connections

    let mut headers = header::HeaderMap::new();

    headers.insert(
        "Origin",
        header::HeaderValue::from_static(ORIGIN),
    );
    headers.insert(
        "Referer",
        header::HeaderValue::from_static(REFERER),
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .default_headers(headers)
        .build()?;

    // 2. Setup Writer
    let final_path = format!("{}/{}/{}.ts", SAVE_PATH, &movie_name, &movie_name);
    let mut file = tokio::fs::File::create(&final_path)
        .await
        .expect("Failed to create file");

    // 3. Setup indicatif Progress Bar
    let total_tasks = (total_frames + 1) as u64;
    let pb = ProgressBar::new(total_tasks);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} segments ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );

    // 4. The Sliding Window Stream
    // We create an iterator of our frame numbers, map them to async download tasks,
    // and let .buffered() handle the concurrency limit and ordering.
    let mut stream = stream::iter(0..=total_frames)
        .map(|i| {
            let client = client.clone();
            let url = format!("{}{}/{}/video{}.jpeg", VIDEO_CDN, uuid, resolution, i);

            // This async block is our worker task
            async move {
                let mut downloaded_bytes: Option<Bytes> = None;

                for _ in 0..3 {
                    if let Ok(resp) = client.get(&url).send().await {
                        if let Ok(bytes) = resp.bytes().await {
                            downloaded_bytes = Some(bytes);
                            break; // Success, break retry loop
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }

                // Return the index and the data (or None if it failed)
                (i, downloaded_bytes)
            }
        })
        .buffered(concurrency_limit); // <-- THIS REPLACES THE SEMAPHORE AND CHANNELS!

    // 5. Consume the stream sequentially
    // Because we used .buffered(), these results are GUARANTEED to arrive in order (0, 1, 2...)
    // even if they finish downloading out of order in the background.
    while let Some((i, result)) = stream.next().await {
        match result {
            Some(data) => {
                file.write_all(&data)
                    .await
                    .expect("Failed to write to file");
                pb.inc(1); // Update progress bar
            }
            None => {
                pb.println(format!("Failed to download frame {} after 3 retries", i));
                return Err(format!("Aborting: Frame {} failed to download", i).into());
            }
        }
    }

    // Flush final bytes to disk
    file.flush().await.expect("Failed to flush file");
    pb.finish_with_message("Download and concatenation complete!");

    Ok(())
}

/*
//Test buffered download and write
use futures::StreamExt; // cargo add futures
use indicatif::{ProgressBar, ProgressStyle}; // cargo add indicatif
use reqwest::Client; // cargo add reqwest -F json,stream
use std::fs::File;
use std::io::{BufWriter, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let total_segments = 500;
    let concurrent_limit = 50; // Your 10% batch limit
    let output_path = "final_video.ts";

    // Setup Progress Bar
    let pb = ProgressBar::new(total_segments as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    // Prepare Output File
    let file = File::create(output_path)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, file); // 1MB Buffer

    // Create the stream logic
    let mut fetches = futures::stream::iter(0..total_segments)
        .map(|i| {
            let client = client.clone();
            async move {
                let url = format!("https://your-server.com/video{}.jpeg", i);
                let res = client.get(&url).send().await?;
                let bytes = res.bytes().await?;
                Ok::<(usize, bytes::Bytes), reqwest::Error>((i, bytes))
            }
        })
        .buffered(concurrent_limit); // This keeps 50 downloads active

    // Process results as they come in (in numerical order)
    while let Some(result) = fetches.next().await {
        let (_id, data) = result?;

        // Write to the single merged file
        writer.write_all(&data)?;

        // Update the UI
        pb.inc(1);
    }

    writer.flush()?;
    pb.finish_with_message("Download and Merge Complete!");

    Ok(())
}
*/

/*
pub async fn download_jpegs_frames_concurrent(
    uuid: &str,
    resolution: &str,
    movie_name: &str,
    total_frames: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Concurrent Download...");
    // let start = Instant::now();
    // 1. Setup shared state
    let client = Arc::new(Client::new());
    let semaphore = Arc::new(Semaphore::new(30)); // Limit to 20 concurrent downloads
    let mut handles = vec![];

    for i in 0..=total_frames {
        let permit = semaphore.clone().acquire_owned().await?;
        let client = Arc::clone(&client);
        let uuid = uuid.to_string();
        let res = resolution.to_string();
        let name = movie_name.to_string();

        let handle = tokio::spawn(async move {
            let url = format!("https://cdn.com/{}/{}/video{}.jpeg", uuid, res, i);
            // let path = format!("{}/{}/video{}.jpeg", SAVE_PATH, name, i);
            let path = format!("{}/{}/video{}.ts", SAVE_PATH, name, i);

            // Retry logic
            let mut downloaded = false;
            for _ in 0..3 {
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Ok(_) = fs::write(&path, bytes) {
                            downloaded = true;
                            break;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }

            if !downloaded {
                eprintln!("Failed to download frame {}", i);
            }

            // CLI Progress Logic
            let count = {
                let mut c = COUNTER.lock().unwrap();
                *c += 1;
                *c
            };
            if count % 50 == 0 || count == total_frames + 1 {
                println!(
                    "Progress: {:.2}% ({}/{})",
                    (count as f32 / (total_frames + 1) as f32) * 100.0,
                    count,
                    total_frames + 1
                );
            }

            drop(permit); // Release slot for next download
        });

        handles.push(handle);
    }

    // Wait for all downloads to finish
    for h in handles {
        let _ = h.await;
    }
    // let elapsed = start.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}

*/

// Sequential Download
/*
async fn download_jpegs_frames(
    intervals: Vec<(i32, i32)>,
    uuid: &str,
    resolution: &str,
    movie_name: &str,
    video_offset_max: i32,
) -> Result<(), String> {
    println!("Download Started...Please wait");
    // let start = Instant::now();
    let total_frames = video_offset_max + 1;
    let mut handles = vec![];

    for (start, end) in intervals {
        let uuid = uuid.to_string();
        let resolution = resolution.to_string();
        let movie_name = movie_name.to_string();

        let handle = tokio::spawn(async move {
            for i in start..end {
                let url_tmp = format!("https://cdn.com/{}/{}/video{}.jpeg", uuid, resolution, i);

                if let Some(content) = request_with_retry(&url_tmp) {
                    let file_path = format!("{}/{}/video{}.jpeg", SAVE_PATH, movie_name, i);
                    if let Some(parent) = Path::new(&file_path).parent() {
                        fs::create_dir_all(parent).expect("Failed to create directories");
                    }

                    if File::create(&file_path)
                        .and_then(|mut file| file.write_all(&content))
                        .is_err()
                    {
                        eprintln!("Failed to write file: {}", file_path);
                        continue;
                    }

                    // CLI Progress Reporting
                    let count = {
                        let mut c = COUNTER.lock().unwrap();
                        *c += 1;
                        *c
                    };

                    if count % 50 == 0 || count == total_frames {
                        println!(
                            "Progress: {:.2}% ({}/{})",
                            (count as f32 / total_frames as f32) * 100.0,
                            count,
                            total_frames
                        );
                    }
                } else {
                    eprintln!("Failed to download: {}", url_tmp);
                }
            }
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    for handle in handles {
        handle
            .await
            .map_err(|e| format!("Thread failed: {:?}", e))??;
    }
    // let elapsed = start.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
    Ok(())
}
*/

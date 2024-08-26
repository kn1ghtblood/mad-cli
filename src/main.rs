use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::{self, Error, Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use ureq;

const SAVE_PATH: &str = "downloads";
const VIDEO_M3U8_PREFIX: &str = "https://surrit.com/";
const VIDEO_PLAYLIST_SUFFIX: &str = "/playlist.m3u8";

struct ThreadSafeCounter {
    count: Mutex<i32>,
}

impl ThreadSafeCounter {
    fn new() -> Self {
        ThreadSafeCounter {
            count: Mutex::new(0),
        }
    }

    fn increment_and_get(&self) -> i32 {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        *count
    }

    fn reset(&self) {
        let mut count = self.count.lock().unwrap();
        *count = 0;
    }

    fn get_count(&self) -> i32 {
        let count = self.count.lock().unwrap();
        *count
    }
}
static COUNTER: Lazy<Arc<ThreadSafeCounter>> = Lazy::new(|| Arc::new(ThreadSafeCounter::new()));

fn get_num() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn display_progress_bar(max_value: i32) {
    let bar_length = 50;
    let current_value = COUNTER.increment_and_get();
    let progress = current_value as f32 / max_value as f32;
    let block = (bar_length as f32 * progress).round() as usize;
    let bar = format!(
        "\rProgress: [{}{}] {}/{}",
        "#".repeat(block),
        "-".repeat(bar_length - block),
        current_value,
        max_value
    );
    print!("{}", bar);
    io::stdout().flush().unwrap();
}

fn thread_task(
    start: i32,
    end: i32,
    uuid: String,
    resolution: String,
    movie_name: String,
    video_offset_max: i32,
) {
    // let movie_save_path_root = "movie_save_path_root";

    for i in start..end {
        let url_tmp = format!("https://surrit.com/{}/{}/video{}.jpeg", uuid, resolution, i);

        if let Some(content) = request_with_retry(&url_tmp) {
            let file_path = format!("{}/{}/video{}.jpeg", SAVE_PATH, movie_name, i);
            if let Some(parent) = Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent).expect("Failed to create directories");
            }

            let mut file = File::create(&file_path).expect("Failed to create file");
            file.write_all(&content)
                .expect("Failed to write content to file");
            // println!("{:?}", file_path);
            display_progress_bar(video_offset_max + 1)
        } else {
            println!("failed");
        }
    }
}

fn frames_to_video(name: &str, total_frames: i32) {
    let out_file_name = format!("{}/{}.mp4", SAVE_PATH, name);
    let mut count = 0;
    let out_path = Path::new(&out_file_name);
    let mut out_file = match OpenOptions::new().write(true).create(true).open(out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create output file: {}", e);
            return;
        }
    };

    for i in 0..=total_frames {
        let file_path = format!("{}/{}/video{}.jpeg", SAVE_PATH, name, i);
        let path = Path::new(&file_path);

        match File::open(path) {
            Ok(mut infile) => {
                let mut buffer = Vec::new();
                if let Err(e) = infile.read_to_end(&mut buffer) {
                    eprintln!("Failed to read from file {}: {}", file_path, e);
                    continue;
                }
                if let Err(e) = out_file.write_all(&buffer) {
                    eprintln!("Failed to write to file {}: {}", out_file_name, e);
                    continue;
                }
                count += 1;
                println!("write: {}", file_path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("file not found: {}", file_path);
                continue;
            }
            Err(e) => {
                eprintln!("exception: {}", e);
                continue;
            }
        }
    }
    let total_files = total_frames + 1;
    let saved_ratio = count as f64 / total_files as f64;

    println!("Save Completed: {}", out_file_name);
    println!(
        "Total number of files: {} , number of files saved: {}",
        total_files, count
    );
    println!("The file integrity is {:.2}%%", saved_ratio);
}

fn gen_list_txt(name: &str, total_frames: i32) -> io::Result<()> {
    let out_file_name = format!("{}/{}.mp4", SAVE_PATH, name);
    let mut count = 0;
    let list_file = format!("{}/{}/list.txt", SAVE_PATH, name);
    let mut list_txt = File::create(list_file)?;

    for i in 0..=total_frames {
        let file_path = format!("{}/{}/video{}.jpeg", SAVE_PATH, name, i);
        let file_name = format!("video{}.jpeg", i);
        if Path::new(&file_path).exists() {
            count += 1;
            writeln!(list_txt, "file '{}'", file_name);
        } else {
            println!("[X]Error locating the frames");
        }
    }
    println!("Complete save jpegs for: {}", out_file_name);
    println!(
        "Total files count: {}, found files count: {}",
        total_frames + 1,
        count
    );
    println!(
        "File integrity is {:.2}%",
        (count as f32 / (total_frames + 1) as f32) * 100.0
    );
    Ok(())
}

fn frames_to_video_ffmpeg(name: &str, total_frames: i32) -> io::Result<()> {
    match gen_list_txt(name, total_frames) {
        Ok(()) => {
            let list_location = format!("{}/{}/list.txt", SAVE_PATH, name);
            let out_file_name = format!("{}/{}.mp4", SAVE_PATH, name);
            //TODO: Use portable FFmpeg binary
            let ffmpeg_path = if cfg!(target_os = "windows") {
                Path::new("bin/ffmpeg.exe")
            } else if cfg!(target_os = "linux") {
                Path::new("bin/ffmpeg")
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    "Unsupported operating system",
                ));
            };
            let ffmpeg_command = [
                // "ffmpeg",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                &list_location,
                "-c",
                "copy",
                &out_file_name,
            ];
            let status = Command::new(ffmpeg_path)
                .args(&ffmpeg_command)
                .stdin(Stdio::null())
                .status()?;

            if status.success() {
                println!("FFmpeg execution completed.");
                Ok(())
            } else {
                eprintln!(
                    "Movie name: {}, FFmpeg execution failed with status: {}",
                    name, status
                );
                Err(Error::new(
                    std::io::ErrorKind::Other,
                    "FFmpeg execution failed",
                ))
            }
        }
        Err(e) => {
            eprintln!("Failed to genearate list: {}", e);
            Err(e)
        }
    }
}

fn get_movie_url_by_code(key: &str) -> Option<String> {
    let search_url = format!("https://missav.com/search/{}", key);
    let search_regex = format!(r#"<a href="([^"]+)" alt="{}">"#, key);

    let response = ureq::get(&search_url)
        .call()
        .expect("Failed to fetch the search URL");
    let html_source = response
        .into_string()
        .expect("Failed to read response body");

    let re = Regex::new(&search_regex).expect("Failed to compile regex");
    let movie_url_matches: Vec<String> = re
        .captures_iter(&html_source)
        .map(|cap| cap[1].to_string())
        .collect();

    let temp_url_list: HashSet<String> = movie_url_matches.into_iter().collect();

    if !temp_url_list.is_empty() {
        Some(temp_url_list.into_iter().next().unwrap())
    } else {
        None
    }
}

fn make_folders(name: &str) {
    let path = format!("{}/{}", &SAVE_PATH, name);
    if !Path::new(&path).exists() {
        match fs::create_dir_all(&path) {
            Ok(_) => println!("Created directory: {}", path),
            Err(e) => eprintln!("Failed to create directory: {}", e),
        }
    } else {
        println!("Directory already exists: {}", path);
    }
}

async fn delete_all_subfolders(folder_path: &str) -> std::io::Result<()> {
    let path = Path::new(folder_path);

    if !path.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let item_path = entry.path();

        if item_path.is_dir() {
            fs::remove_dir_all(item_path)?;
        }
    }

    Ok(())
}

fn split_integer_into_intervals(integer: i32, n: usize) -> Vec<(i32, i32)> {
    let interval_size = integer / n as i32;
    let remainder = integer % n as i32;

    let mut intervals: Vec<(i32, i32)> = (0..n)
        .map(|i| (i as i32 * interval_size, (i as i32 + 1) * interval_size))
        .collect();

    if let Some(last) = intervals.last_mut() {
        last.1 += remainder;
    }
    intervals
}

fn request_with_retry(url: &str) -> Option<Vec<u8>> {
    let max_retries = 5;
    let delay = 2;
    let mut retries = 0;
    while retries < max_retries {
        match ureq::get(url).call() {
            Ok(res) => {
                if res.status() == 200 {
                    // return Some(res.into_reader().bytes().collect::<Result<Vec<_>, _>>());
                    let mut reader = res.into_reader();
                    let mut bytes = Vec::new();
                    match reader.read_to_end(&mut bytes) {
                        Ok(_) => return Some(bytes),
                        Err(e) => println!("Failed to read response into bytes: {}", e),
                    }
                }
            }
            Err(_) => {
                retries += 1;
                sleep(Duration::from_secs(delay));
            }
        }
    }
    None
}

fn download_jpegs_frames(
    intervals: Vec<(i32, i32)>,
    uuid: String,
    resolution: String,
    movie_name: String,
    video_offset_max: i32,
) -> Result<(), String> {
    let mut thread_task_list = vec![];

    for interval in intervals {
        let uuid_clone = uuid.clone();
        let resolution_clone = resolution.clone();
        let movie_name_clone = movie_name.clone();
        let start = interval.0;
        let end = interval.1;
        let video_offset_max_clone = video_offset_max;

        let thread = thread::spawn(move || {
            thread_task(
                start,
                end,
                uuid_clone,
                resolution_clone,
                movie_name_clone,
                video_offset_max_clone,
            );
        });

        thread_task_list.push(thread);
    }

    for thread in thread_task_list {
        if let Err(e) = thread.join() {
            eprintln!("Thread failed: {:?}", e);
            return Err(format!("Thread faile: {:?}", e));
        }
    }
    Ok(())
}

async fn get_uuid(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = ureq::get(&url).call()?.into_string()?;
    // println!("response: {}", res);
    let re = Regex::new(r"https:\\/\\/sixyik\.com\\/([^\\/]+)\\/seek\\/_0\.jpg")?;
    if let Some(captures) = re.captures(&res) {
        // Extract and return the UUID
        let uuid = captures.get(1).map_or("", |m| m.as_str());
        println!("Matching uuid successfully: {}", uuid);
        Ok(uuid.to_string())
    } else {
        eprintln!("Failed to match uuid.");
        Err("Failed to match uuid.".into())
    }
}

async fn download(url: &str) {
    match get_uuid(url).await {
        Ok(uuid) => {
            // println!("{}", uuid);
            let playlist_url = format!("{}{}{}", VIDEO_M3U8_PREFIX, uuid, VIDEO_PLAYLIST_SUFFIX);
            // println!("{}", playlist_url);
            match ureq::get(&playlist_url).call() {
                Ok(response) => match response.into_string() {
                    Ok(playlist) => {
                        // println!("{}", playlist);
                        let lines: Vec<&str> = playlist.lines().collect();
                        let last_line = lines.last().unwrap();
                        let resolution = last_line.split('/').next().unwrap();
                        println!("{}", resolution);
                        let m3u8_url = format!("{}{}/{}", VIDEO_M3U8_PREFIX, uuid, last_line);
                        // println!("{}", m3u8_url);
                        match ureq::get(&m3u8_url).call() {
                            Ok(res) => match res.into_string() {
                                Ok(off_max_str) => {
                                    let lines: Vec<&str> = off_max_str.lines().collect();
                                    if lines.len() >= 2 {
                                        let off_max = lines[lines.len() - 2];
                                        let re = Regex::new(r"\d+").unwrap();
                                        if let Some(captures) = re.captures(&off_max) {
                                            if let Some(matched) = captures.get(0) {
                                                if let Ok(digit) = matched.as_str().parse::<i32>() {
                                                    println!("Extracted count: {}", digit);
                                                    let movie_name =
                                                        url.rsplit('/').next().unwrap();
                                                    // println!("{}", movie_name);
                                                    make_folders(movie_name);
                                                    let num_cpus = get_num();
                                                    // println!("no cpus: {}", num_cpus);
                                                    let intervals = split_integer_into_intervals(
                                                        digit + 1,
                                                        num_cpus,
                                                    );
                                                    COUNTER.reset();
                                                    if download_jpegs_frames(
                                                        intervals,
                                                        uuid,
                                                        resolution.to_string(),
                                                        movie_name.to_string(),
                                                        digit,
                                                    )
                                                    .is_ok()
                                                    {
                                                        let file_path = format!(
                                                            "{}/{}.mp4",
                                                            SAVE_PATH, movie_name
                                                        );
                                                        if !Path::new(&file_path).exists() {
                                                            // frames_to_video(movie_name, digit);
                                                            frames_to_video_ffmpeg(
                                                                movie_name, digit,
                                                            );
                                                        }
                                                    }
                                                    COUNTER.reset();
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to read response body: {}", e)
                                }
                            },
                            Err(e) => {
                                println!("Failed to read response body: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to read response body: {}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to make request: {}", e);
                }
            }
        }
        Err(e) => println!("Failed to get UUID: {}", e),
    }
}

async fn init_download(urls: Vec<String>, jcode: Option<String>) {
    if !urls.is_empty() {
        println!("URLs provided: {:?}", urls);
        for url in urls {
            println!("[+]Processing URL: {}", url.clone());
            delete_all_subfolders(SAVE_PATH).await;
            download(&url).await;
            // delete_all_subfolders(SAVE_PATH).await;
            println!("[+]Processing URL Complete: {}", url);
        }
    } else if let Some(s) = jcode {
        println!("Code provided: {}", s);
        match get_movie_url_by_code(&s) {
            Some(url) => {
                download(&url).await;
            }
            None => println!("Video URL not found for code: {}", s),
        }
    } else {
        println!("No valid argument provided.");
    }
}

#[tokio::main]
async fn main() {
    println!("[!]Starting Execution");
    let args: Vec<String> = env::args().collect();
    let mut urls: Vec<String> = Vec::new();
    let mut jcode: Option<String> = None;

    let mut args_iter = args.iter().peekable();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-u" => {
                while let Some(value) = args_iter.peek() {
                    if value.starts_with('-') {
                        break;
                    }
                    urls.push(value.to_string());
                    args_iter.next();
                }
            }
            "-s" => {
                if jcode.is_none() && urls.is_empty() {
                    if let Some(value) = args_iter.next() {
                        jcode = Some(value.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    init_download(urls, jcode).await;

    // let url = "";
    // download(url).await;
}

use crate::downloader::download_and_concat_hls;
use crate::media::media_metadata_fix;
use crate::network;
use crate::utils::{cleanup_seg, clog, make_folders, LogLvl};
use crate::config::{SAVE_PATH, VIDEO_CDN, VIDEO_PLAYLIST_SUFFIX, ORIGIN, REFERER};
use dialoguer::{theme::ColorfulTheme, Select};
use regex::Regex;
use std::path::Path;

pub async fn download(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Fetching Information...");
    clog(LogLvl::Info, "Fetching Information...");
    let uuid = network::get_uuid(&url).await?;
    let playlist_url = format!("{}{}{}", VIDEO_CDN, uuid, VIDEO_PLAYLIST_SUFFIX);
    // println!("playlist_url {}", playlist_url);
    let playlist = ureq::get(&playlist_url)
        .set("Origin", ORIGIN)
        .set("Referer", REFERER)
        .call()?
        .into_string()?;
    // println!("playlist {}", playlist);

    // let re = Regex::new(r#"#EXT-X-STREAM-INF:BANDWIDTH=\d+,CODECS="[^"]+",RESOLUTION=(\d+x\d+)"#)
    //     .unwrap();

    // let re = Regex::new(r"#EXT-X-STREAM-INF:.*RESOLUTION=(\d+x\d+)").unwrap();

    // prints list of all the strings scrapped
    // let lines: Vec<&str> = playlist.lines().collect();
    // println!("lines vec {:?}", &lines);

    // let resolutions: Vec<String> = re
    //     .captures_iter(&playlist)
    //     .map(|caps| caps[1].to_string())
    //     .collect();
    // println!("Available Resolutions: {:?}", &resolutions);

    let re = Regex::new(r"(\d{3,4}x\d{3,4}|\d{3,4}p)/video\.m3u8").unwrap();

    let my_res: Vec<String> = re
        .captures_iter(&playlist)
        .map(|caps| caps[1].to_string())
        .collect();
    // println!("Available Resolutions: {:?}", &my_res);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick the resolution")
        .default(0)
        .items(&my_res[..])
        .interact()
        .unwrap();
    // println!("Selected Resolution {}", &my_res[selection]);

    //This fetches the max res ie, last res found on the string
    // let resolution = lines.last().unwrap().split('/').next().unwrap();
    // println!("auto res {}", resolution);

    // let m3u8_url = format!("{}{}/{}", VIDEO_CDN, uuid, lines.last().unwrap());
    let m3u8_url = format!(
        "{}{}/{}/video.m3u8",
        VIDEO_CDN, uuid, &my_res[selection]
    );
    // println!("m3u8_url {}", m3u8_url);

    let off_max_str = ureq::get(&m3u8_url)
        .set("Origin", ORIGIN)
        .set("Referer", REFERER)
        .call()?
        .into_string()?;

    let lines: Vec<&str> = off_max_str.lines().collect();
    // println!("off_max_str {}", off_max_str);
    let off_max = lines[lines.len() - 2];

    // let off_max = lines
    //     .iter()
    //     .filter(|l| !l.starts_with('#') && !l.is_empty())
    //     .last()
    //     .ok_or("Could not find segment count line")?;

    let re = Regex::new(r"\d+")?;
    let digit = re
        .captures(off_max)
        .and_then(|captures| captures.get(0))
        .and_then(|matched| matched.as_str().parse::<i32>().ok())
        .ok_or("Failed to extract count")?;

    let movie_name = url
        .rsplit('/')
        .next()
        .ok_or("Could not extract movie name from URL")?
        .to_string();

    make_folders(&movie_name)?;

    /*
    let num_cpus = get_num_cpus();
    let intervals = split_integer_into_intervals(digit + 1, num_cpus);

    reset_counter();

    let result = download_jpegs_frames(
        intervals,
        &uuid,
        resolution,
        &movie_name,
        digit,
    )
    .await;
    */

    // let res_path: String = if my_res[selection].contains('x') {
    //     my_res[selection]
    //         .split('x')
    //         .last()
    //         .map(|s| format!("{}p", s))
    //         .ok_or("")
    // } else if my_res[selection].ends_with('p') {
    //     Ok(my_res[selection].to_string())
    // } else {
    //     Err("Resolution check failed")
    // }
    // .unwrap_or_else(|_| "720p".to_string());

    let res_path = match &my_res[selection] {
        s if s.contains('x') => s
            .split('x')
            .last()
            .map(|v| format!("{}p", v))
            .unwrap_or("720p".into()),
        s if s.ends_with('p') => s.to_string(),
        _ => "720p".to_string(),
    };

    // println!("{}", res_path);

    let file_path = format!(
        "{}/{}/{}_{}.mp4",
        SAVE_PATH, &movie_name, &movie_name, &res_path
    );

    if Path::new(&file_path).exists() {
        println!("File already exists, skipping download.");
        return Ok(());
    }

    let result = download_and_concat_hls(
        &uuid,
        &my_res[selection], //resolution in the uri
        &movie_name,
        digit as usize,
        20 as usize,
    )
    .await;

    if result.is_ok() {
        // println!("calling postfixer");
        let seg_path = format!("{}/{}/{}.ts", SAVE_PATH, &movie_name, &movie_name);
        if media_metadata_fix(&seg_path, &file_path).is_ok() {
            // println!("no error during media patching, initiate cleanup");
            cleanup_seg(&seg_path)
        }
    }

    // reset_counter();
    Ok(())
}

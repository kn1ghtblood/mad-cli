// use crate::{COUNTER,SAVE_PATH};
use crate::config::SAVE_PATH;
use std::{fs, io};
// use std::path::Path;

pub fn make_folders(name: &str) -> io::Result<()> {
    let path = format!("{}/{}", SAVE_PATH, name);
    fs::create_dir_all(&path)?;
    // println!("Created directory: {}", path);
    clog(LogLvl::Info, format!("Created directory: {}", path).as_str());
    Ok(())
}


pub fn cleanup_seg(seg_path: &str) {
    match fs::remove_file(seg_path) {
        // Ok(_) => println!("Segment artifact cleaned up."),
        Ok(_) => clog(LogLvl::Info, "Segment artifact cleaned up."),
        Err(e) => match e.kind() {
            // io::ErrorKind::NotFound => println!("File already gone, nothing to do."),
            io::ErrorKind::NotFound => clog(LogLvl::Error, "File already gone, nothing to do."),
            io::ErrorKind::PermissionDenied => eprintln!("Error: Permission denied."),
            _ => eprintln!("An error occurred: {}", e),
        },
    }
}
pub enum LogLvl { Error, Warn, Info, Debug }

pub fn clog(log_lvl: LogLvl, log_msg: &str) {
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";
    match log_lvl{
        LogLvl::Error => println!("{}[!] ERROR: {}{}", RED, log_msg, RESET),
        LogLvl::Warn => println!("{}[?] WARN:  {}{}", YELLOW, log_msg, RESET),
        LogLvl::Info => println!("{}[+] INFO:  {}{}", GREEN, log_msg, RESET),
        LogLvl::Debug => println!("{}[d] DEBUG: {}{}", CYAN, log_msg, RESET),
        // _ => println!("[*] LOG:   {}", log_msg),
    }
}

// pub fn delete_all_subfolders(folder_path: &str) -> std::io::Result<()> {
//     let _path = Path::new(folder_path);
//     // if !path.exists() {
//     //     return Ok(());
//     // }

//     // for entry in fs::read_dir(path)? {
//     //     let entry = entry?;
//     //     let item_path = entry.path();

//     //     if item_path.is_dir() {
//     //         fs::remove_dir_all(item_path)?;
//     //     }
//     // }
//     Ok(())
// }

// pub fn reset_counter() {
//     if let Ok(mut count) = COUNTER.lock() {
//         *count = 0;
//     }
// }

// pub fn debug_save(digit) {
//     println!("Video processing started...Please wait");
//     let list_file = format!("{}/{}/list.txt", SAVE_PATH, &movie_name);
//     let mut list_txt = File::create(&list_file)?;

//     for i in 0..=digit {
//         let file_path = format!("{}/{}/video{}.ts", SAVE_PATH, &movie_name, i);
//         if Path::new(&file_path).exists() {
//             // writeln!(list_txt, "{}/{}/video{}.ts", SAVE_PATH, &movie_name, i)?;
//             writeln!(list_txt, "file 'video{}.ts'", i)?;
//         }
//     }
// }

// pub fn get_num_cpus() -> usize {
//     std::thread::available_parallelism()
//         .map(|n| n.get())
//         .unwrap_or(1)
// }

// pub fn split_integer_into_intervals(integer: i32, n: usize) -> Vec<(i32, i32)> {
//     let interval_size = integer / n as i32;
//     let remainder = integer % n as i32;

//     let mut intervals: Vec<(i32, i32)> = (0..n)
//         .map(|i| (i as i32 * interval_size, (i as i32 + 1) * interval_size))
//         .collect();

//     if let Some(last) = intervals.last_mut() {
//         last.1 += remainder;
//     }
//     intervals
// }

// fn ffmpeg_check() -> bool {
//     Command::new("ffmpeg")
//         .arg("-version")
//         .output()
//         .map_or(false, |output| output.status.success())
// }

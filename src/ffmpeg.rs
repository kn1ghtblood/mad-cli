use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};

use crate::config::SAVE_PATH;

pub fn gen_list_txt(name: &str, total_frames: i32) -> io::Result<()> {
    let list_file = format!("{}/{}/list.txt", SAVE_PATH, name);
    let mut list_txt = File::create(list_file)?;

    for i in 0..=total_frames {
        let file_name = format!("video{}.jpeg", i);
        writeln!(list_txt, "file '{}'", file_name)?;
    }

    Ok(())
}

pub fn frames_to_video_ffmpeg(name: &str, total_frames: i32) -> io::Result<()> {
    gen_list_txt(name, total_frames)?;

    let list_location = format!("{}/{}/list.txt", SAVE_PATH, name);
    let out_file_name = format!("{}/{}.mp4", SAVE_PATH, name);

    let ffmpeg_path = if cfg!(target_os = "windows") {
        Path::new("bin/ffmpeg.exe")
    } else {
        Path::new("bin/ffmpeg")
    };

    Command::new(ffmpeg_path)
        .args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &list_location,
            "-c",
            "copy",
            &out_file_name,
        ])
        .stdin(Stdio::null())
        .status()?;

    Ok(())
}
// fn frames_to_video_ffmpeg(name: &str, total_frames: i32) -> io::Result<()> {
//     println!("FFMPEG Detected.\nVideo processing started...Please wait");
//     let list_file = format!("{}/{}/list.txt", SAVE_PATH, name);
//     let mut list_txt = File::create(&list_file)?;

//     for i in 0..=total_frames {
//         let file_path = format!("{}/{}/video{}.jpeg", SAVE_PATH, name, i);
//         if Path::new(&file_path).exists() {
//             writeln!(list_txt, "file 'video{}.jpeg'", i)?;
//         }
//     }

//     let out_file_name = format!("{}/{}.mp4", SAVE_PATH, name);

//     let ffmpeg_path = if cfg!(target_os = "windows") {
//         "ffmpeg.exe"
//     } else if cfg!(target_os = "linux") {
//         "ffmpeg"
//     } else {
//         return Err(io::Error::new(
//             io::ErrorKind::Other,
//             "Unsupported operating system",
//         ));
//     };

//     let status = Command::new(ffmpeg_path)
//         .args([
//             "-f",
//             "concat",
//             "-safe",
//             "0",
//             "-i",
//             &list_file,
//             "-c",
//             "copy",
//             &out_file_name,
//         ])
//         .stdin(Stdio::null())
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .status()?;

//     if status.success() {
//         match delete_all_subfolders(SAVE_PATH) {
//             Ok(_) => println!("successfully deleted temp files"),
//             Err(e) => eprintln!("{}", e),
//         }
//         println!("FFmpeg execution completed.");
//         println!("SUCCESS!!! Output Saved to : {}", SAVE_PATH);
//         Ok(())
//     } else {
//         Err(io::Error::new(
//             io::ErrorKind::Other,
//             format!("FFmpeg execution failed for movie: {}", name),
//         ))
//     }
// }
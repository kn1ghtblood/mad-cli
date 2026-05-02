use std::env;
use std::path::PathBuf;

fn main() {
    // Get the directory where Cargo.toml is located
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Detect the TARGET OS (not the HOST OS)
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Library subfolder
    let os_folder = match target_os.as_str() {
        "windows" => "windows",
        "linux" => "linux",
        "android" => "android",
        _ => panic!("Unsupported target OS: {}", target_os),
    };

    // Construct paths relative to the project root
    let ffmpeg_base = manifest_dir.join("libs").join(os_folder);
    let lib_dir = ffmpeg_base.join("lib");
    let include_dir = ffmpeg_base.join("include");

    // Tell Cargo where to find the library files (.lib, .a, or .so)
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Export the include path (useful if you use bindgen or cc)
    println!("cargo:include={}", include_dir.display());

    // Force static linking for FFmpeg components
    println!("cargo:rustc-link-lib=static=avformat");
    println!("cargo:rustc-link-lib=static=avcodec");
    println!("cargo:rustc-link-lib=static=avutil");
    // println!("cargo:rustc-link-lib=static=avdevice");
    // println!("cargo:rustc-link-lib=static=avfilter");

    // Link System-Specific Dependencies
    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=user32");
            println!("cargo:rustc-link-lib=dylib=bcrypt");
            println!("cargo:rustc-link-lib=dylib=ole32");
            println!("cargo:rustc-link-lib=dylib=advapi32");
            println!("cargo:rustc-link-lib=dylib=shell32");
            println!("cargo:rustc-link-lib=dylib=mfplat");
            println!("cargo:rustc-link-lib=dylib=mfuuid");
            println!("cargo:rustc-link-lib=dylib=strmiids");
            println!("cargo:rustc-link-lib=dylib=uuid");
            println!("cargo:rustc-link-lib=dylib=gdi32");
            println!("cargo:rustc-link-lib=dylib=vfw32");
            println!("cargo:rustc-link-lib=dylib=shlwapi");
            println!("cargo:rustc-link-lib=dylib=oleaut32");
            // println!("cargo:rustc-link-lib=mfplat");
            // println!("cargo:rustc-link-lib=mfuuid");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=z");
            // println!("cargo:rustc-link-lib=dylib=lzma");
            // println!("cargo:rustc-link-lib=dylib=m");
            // println!("cargo:rustc-link-lib=dylib=pthread");
        }
        "android" => {
            println!("cargo:rustc-link-lib=dylib=z");
            // println!("cargo:rustc-link-lib=dylib=m");
            // println!("cargo:rustc-link-lib=dylib=log");
            // println!("cargo:rustc-link-lib=dylib=android");
        }
        _ => {}
    }

    // Ensure Cargo reruns this if you swap out the library files
    println!("cargo:rerun-if-changed=libs/{}", os_folder);
    println!("cargo:rerun-if-changed=build.rs");
}

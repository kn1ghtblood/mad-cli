use ffmpeg_next as ffmpeg;
use std::path::Path;
use crate::utils::{clog, LogLvl};

pub fn media_metadata_fix(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()?;
    // Optional: Silence the "Starting second pass" logs
    // ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Info);
    ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Quiet);
    // println!("in postfixer, in={} out={}", &input_path, &output_path);
    // let input_path = "test.mp4";
    // let output_path = "final.mp4";

    if !Path::new(input_path).exists() {
        eprintln!("Error: input file doesn't exists.");
        std::process::exit(1);
    }

    if Path::new(output_path).exists() {
        eprintln!("Error: output file already exists.");
        std::process::exit(1);
    }

    let mut ictx = ffmpeg::format::input(&input_path)?;
    let mut octx = ffmpeg::format::output(&output_path)?;

    let mut stream_mapping: Vec<i32> = vec![-1; ictx.nb_streams() as usize];
    let mut out_index = 0;

    for ist in ictx.streams() {
        let medium = ist.parameters().medium();
        if !matches!(
            medium,
            ffmpeg::media::Type::Audio | ffmpeg::media::Type::Video | ffmpeg::media::Type::Subtitle
        ) {
            continue;
        }

        stream_mapping[ist.index()] = out_index;

        // Add stream
        let mut ost = octx.add_stream(ffmpeg::encoder::find(ffmpeg::codec::Id::None))?;
        ost.set_parameters(ist.parameters());

        unsafe {
            // Reset codec_tag so the muxer picks the correct one for MP4
            (*ost.parameters().as_mut_ptr()).codec_tag = 0;
        }
        out_index += 1;
    }

    // --- Handle Global Headers and Write Header ---
    unsafe {
        let ofmt_ptr = (*octx.as_ptr()).oformat;
        if !ofmt_ptr.is_null() && ((*ofmt_ptr).flags & ffmpeg::ffi::AVFMT_GLOBALHEADER as i32 != 0)
        {
            // Set the flag on the Output Format Context itself
            (*octx.as_mut_ptr()).flags |= ffmpeg::ffi::AVFMT_GLOBALHEADER as i32;
        }
    }

    let mut mux_opts = ffmpeg::Dictionary::new();
    mux_opts.set("movflags", "faststart");
    octx.write_header_with(mux_opts)?;

    // FIX 3: Pre-collect timebases and start offsets for performance and sync
    let out_time_bases: Vec<_> = octx.streams().map(|s| s.time_base()).collect();
    //let in_start_times: Vec<_> = ictx.streams().map(|s| s.start_time().unwrap_or(0)).collect();
    let in_start_times: Vec<i64> = ictx
        .streams()
        .map(|s| {
            let start = s.start_time();
            if start == i64::MIN {
                0
            } else {
                start
            } // i64::MIN is often used for AV_NOPTS_VALUE
        })
        .collect();

    for (ist, mut packet) in ictx.packets() {
        let ist_idx = ist.index();
        let ost_idx = stream_mapping[ist_idx];

        if ost_idx < 0 {
            continue;
        }

        let ost_idx_usize = ost_idx as usize;

        // FIX 4: Normalize timestamps to fix Audio/Video seek desync
        let start_offset = in_start_times[ist_idx];
        packet.set_pts(packet.pts().map(|p| p - start_offset));
        packet.set_dts(packet.dts().map(|d| d - start_offset));

        // FIX 5: Rescale to the output container's timebase
        packet.rescale_ts(ist.time_base(), out_time_bases[ost_idx_usize]);
        packet.set_position(-1);
        packet.set_stream(ost_idx_usize);

        packet.write_interleaved(&mut octx)?;
    }

    octx.write_trailer()?;
    // println!("Remuxing complete: moov atom moved to start.");
    clog(LogLvl::Info, "Remuxing complete: moov atom moved to start.");
    Ok(())
}

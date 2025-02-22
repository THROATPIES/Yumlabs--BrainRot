use std::{
    fs::File,
    path::Path,
    process::Command,
};
use symphonia::core::{
    codecs::CODEC_TYPE_NULL,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

const PYTHON_ENCODING: &str = "utf8";
type ResultString<T> = Result<T, String>;

struct VideoGeneratorConfig<'a> {
    video_clip_path: &'a str,
    audio_clip_path: &'a str,
    formatted_text: &'a str,
    output_video_path: &'a str,
    subtitle_fontsize: Option<i32>,
    subtitle_color: Option<&'a str>,
}

fn execute_command(cmd: &mut Command) -> ResultString<()> {
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute video generation script: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Video generation failed: {}", error))
    }
}

pub fn get_duration_from_audio(audio_clip_path: &str) -> ResultString<f32> {
    let file = open_audio_file(audio_clip_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let format = probe_audio_format(mss)?;
    let track = find_audio_track(&format)?;
    let duration = calculate_duration(&track)?;
    
    Ok(duration as f32)
}

fn open_audio_file(path: &str) -> ResultString<File> {
    File::open(Path::new(path))
        .map_err(|e| format!("Failed to open audio file: {}", e))
}

fn probe_audio_format(mss: MediaSourceStream) -> ResultString<Box<dyn symphonia::core::formats::FormatReader>> {
    let mut hint = Hint::new();
    hint.with_extension("wav");

    symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("Probe error: {}", e))
        .map(|probed| probed.format)
}

fn find_audio_track(format: &Box<dyn symphonia::core::formats::FormatReader>) -> ResultString<symphonia::core::formats::Track> {
    format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks".to_string())
        .map(|track| track.clone())
}

fn calculate_duration(track: &symphonia::core::formats::Track) -> ResultString<f64> {
    track.codec_params.time_base
        .and_then(|time_base| track.codec_params.n_frames
            .map(|frames| (frames as f64 * time_base.numer as f64) / time_base.denom as f64))
        .ok_or("Could not calculate duration".to_string())
}

pub fn execute_python_video_generator(
    video_clip_path: &str,
    audio_clip_path: &str,
    formatted_text: &str,
    output_video_path: &str,
    subtitle_fontsize: Option<i32>,
    subtitle_color: Option<&str>,
) -> ResultString<()> {
    let config = VideoGeneratorConfig {
        video_clip_path,
        audio_clip_path,
        formatted_text,
        output_video_path,
        subtitle_fontsize,
        subtitle_color,
    };
    
    let mut cmd = build_python_command(&config);
    execute_command(&mut cmd)
}

fn build_python_command(config: &VideoGeneratorConfig) -> Command {
    let mut cmd = Command::new("python");
    cmd.env("PYTHONIOENCODING", PYTHON_ENCODING)
        .arg("src/vid_generator.py")
        .arg(config.video_clip_path)
        .arg(config.audio_clip_path)
        .arg(config.formatted_text.replace("'", "\\'"))
        .arg(config.output_video_path);

    if let Some(size) = config.subtitle_fontsize {
        cmd.arg(size.to_string());
        if let Some(color) = config.subtitle_color {
            cmd.arg(color);
        }
    }

    cmd
}

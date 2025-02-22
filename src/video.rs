use std::process::Command;
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

const PYTHON_ENCODING: &str = "utf8";

type ResultString<T> = Result<T, String>;

fn execute_command(cmd: &mut Command) -> ResultString<()> {
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Video generation failed: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute video generation script: {}", e)),
    }
}

pub fn get_duration_from_audio(audio_clip_path: &str) -> Result<f32, String> {
    let file = File::open(Path::new(audio_clip_path))
        .map_err(|e| format!("Failed to open audio file: {}", e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("wav");

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| format!("Probe error: {}", e))?;

    let format = probed.format;

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks")?;

    let duration = track.codec_params.time_base
        .and_then(|time_base| track.codec_params.n_frames
            .map(|frames| (frames as f64 * time_base.numer as f64) / time_base.denom as f64))
        .ok_or("Could not calculate duration")?;

    Ok(duration as f32)
}

pub fn execute_python_video_generator(
    video_clip_path: &str,
    audio_clip_path: &str,
    formatted_text: &str,
    output_video_path: &str,
    subtitle_fontsize: Option<i32>,
    subtitle_color: Option<&str>,
) -> ResultString<()> {
    let mut cmd = Command::new("python");
    cmd.env("PYTHONIOENCODING", PYTHON_ENCODING)
        .arg("src/vid_generator.py")
        .arg(video_clip_path)
        .arg(audio_clip_path)
        .arg(formatted_text.replace("'", "\\'"))
        .arg(output_video_path);

    if let Some(size) = subtitle_fontsize {
        cmd.arg(size.to_string());
        if let Some(color) = subtitle_color {
            cmd.arg(color);
        }
    }

    execute_command(&mut cmd)
}

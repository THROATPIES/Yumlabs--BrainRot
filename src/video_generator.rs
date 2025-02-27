use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Context, Result};

/// Uses ffprobe to get the duration (in seconds) of the audio file.
fn get_audio_duration(audio_path: &str) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            audio_path,
        ])
        .output()
        .context("Failed to execute ffprobe")?;
    if !output.status.success() {
        return Err(anyhow!("ffprobe command failed"));
    }
    let duration_str = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    let duration: f64 = duration_str.parse().context("Failed to parse duration")?;
    Ok(duration)
}

/// Converts seconds into ASS time format ("H:MM:SS.CS").
fn seconds_to_ass_time(seconds: f64) -> String {
    let total_seconds = seconds as i64;
    let centiseconds = ((seconds - (total_seconds as f64)) * 100.0) as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    format!("{:01}:{:02}:{:02}.{:02}", hours, minutes, secs, centiseconds)
}

/// Generates an .ass subtitle file content using the formatted text.
/// Here each word is shown for an equal fraction of the audio duration.
/// The style includes an outline (stroke) effect similar to your Python code.
fn generate_subtitle_ass(
    formatted_text: &str,
    audio_duration: f64,
    subtitle_fontsize: i32,
    subtitle_color: &str,
    video_size: (i32, i32),
    _font_path: &str, // provided for similarity – in ASS the font name is used instead of the path
) -> String {
    let words: Vec<&str> = formatted_text.split_whitespace().collect();
    let num_words = words.len();
    let word_duration = if num_words > 0 {
        audio_duration / (num_words as f64)
    } else {
        0.0
    };

    // Create the ASS header
    let mut ass = String::new();
    ass.push_str("[Script Info]\n");
    ass.push_str("ScriptType: v4.00+\n");
    ass.push_str(&format!("PlayResX: {}\nPlayResY: {}\n\n", video_size.0, video_size.1));

    // Define one style called "Default" with a border for the outline.
    // Note: ASS colors are in &HBBGGRR format. For simplicity, we assume "white" and "black".
    let primary_color = if subtitle_color.to_lowercase() == "white" {
        "&H00FFFFFF"
    } else {
        "&H000000" // fallback
    };

    ass.push_str("[V4+ Styles]\n");
    ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, \
                Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, \
                Alignment, MarginL, MarginR, MarginV, Encoding\n");
    // Using border style 1 (outline), Outline thickness 2, no shadow.
    ass.push_str(&format!(
        "Style: Default,Roboto-Bold,{},{} ,{},&H000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n",
        subtitle_fontsize, primary_color, primary_color
    ));

    // [Events] section with one dialogue line per word.
    ass.push_str("[Events]\n");
    ass.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let mut current_time = 0.0;
    for word in words {
        let start = seconds_to_ass_time(current_time);
        let end = seconds_to_ass_time(current_time + word_duration);
        // Fixing escaped quotes as in the original code.
        let text = word.replace("\\'", "'");
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0000,0000,0000,,{}\n",
            start, end, text
        ));
        current_time += word_duration;
    }
    ass
}

/// Generates the final video by overlaying subtitles and merging audio.
/// This function mimics the Python code functionality using ffmpeg.
fn generate_video(
    video_clip_path: &str,
    audio_clip_path: &str,
    formatted_text: &str,
    output_video_path: &str,
    subtitle_fontsize: i32,
    subtitle_color: &str,
) -> Result<()> {
    // Check that input files exist.
    if !Path::new(video_clip_path).exists() || !Path::new(audio_clip_path).exists() {
        return Err(anyhow!("Input files not found"));
    }

    // Get the duration of the audio clip.
    let audio_duration = get_audio_duration(audio_clip_path)?;

    // For simplicity, assume a fixed video resolution (or obtain it via ffprobe).
    let video_size = (1280, 720);

    // Generate the subtitle file content.
    let ass_content = generate_subtitle_ass(
        formatted_text,
        audio_duration,
        subtitle_fontsize,
        subtitle_color,
        video_size,
        "data/inputs/Roboto-Bold.ttf",
    );
    let ass_file_path = "temp_subtitles.ass";
    {
        let mut file = File::create(ass_file_path)
            .context("Failed to create temporary subtitles file")?;
        file.write_all(ass_content.as_bytes())
            .context("Failed to write subtitles file")?;
    }

    // Build the ffmpeg command.
    // This command:
    //   - Loads the video and audio.
    //   - Overlays subtitles using the generated .ass file.
    //   - Uses libx264 with preset ultrafast and 24 fps.
    //   - Scales the audio volume to 0.8.
    //   - Trims the output to the audio duration.
    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i", video_clip_path,
            "-i", audio_clip_path,
            "-vf", &format!("subtitles={}", ass_file_path),
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-t", &audio_duration.to_string(),
            "-r", "24",
            "-c:a", "aac",
            "-b:a", "192k",
            "-filter:a", "volume=0.8",
            "-shortest",
            output_video_path,
        ])
        .status()
        .context("Failed to execute ffmpeg")?;

    // Clean up the temporary subtitles file.
    let _ = std::fs::remove_file(ass_file_path);

    if !status.success() {
        return Err(anyhow!("ffmpeg command failed"));
    }
    Ok(())
}

pub fn generate_video_from_args() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} video_clip_path audio_clip_path formatted_text output_video_path [font] [fontsize] [color]", args[0]);
        std::process::exit(1);
    }
    let video_clip_path = &args[1];
    let audio_clip_path = &args[2];
    let formatted_text = &args[3];
    let output_video_path = &args[4];
    // The optional arguments for font are ignored here.
    let subtitle_fontsize = if args.len() > 6 {
        args[6].parse().unwrap_or(24)
    } else {
        24
    };
    let subtitle_color = if args.len() > 7 {
        &args[7]
    } else {
        "white"
    };

    match generate_video(
        video_clip_path,
        audio_clip_path,
        formatted_text,
        output_video_path,
        subtitle_fontsize,
        subtitle_color,
    ) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error generating video: {}", e);
            std::process::exit(1)
        }
    }
}

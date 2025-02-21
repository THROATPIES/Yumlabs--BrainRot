use std::process::Command;

const SCRIPT_PATH: &str = "src/vid_generator.py";
const PYTHON_ENCODING: &str = "utf8";

type ResultString<T> = Result<T, String>;

pub fn generate_video(
    video_clip_path: &str,
    audio_clip_path: &str,
    formatted_text: &str,
    output_video_path: &str,
    subtitle_fontsize: Option<i32>,
    subtitle_color: Option<&str>,
) -> ResultString<()> {
    let mut cmd = build_command(
        video_clip_path,
        audio_clip_path,
        formatted_text,
        output_video_path,
        subtitle_fontsize,
        subtitle_color,
    );

    execute_command(&mut cmd)
}


fn build_command(
    video_clip_path: &str,
    audio_clip_path: &str,
    formatted_text: &str,
    output_video_path: &str,
    subtitle_fontsize: Option<i32>,
    subtitle_color: Option<&str>,
) -> Command {
    let mut cmd = Command::new("python");
    cmd.env("PYTHONIOENCODING", PYTHON_ENCODING)
        .arg(SCRIPT_PATH)
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

    
    println!("Video clip path: {}", video_clip_path);
    println!("Audio clip path: {}", audio_clip_path);
    println!("Output path: {}", output_video_path);
    println!("Font size: {:?}", subtitle_fontsize);
    println!("Color: {:?}", subtitle_color);

    cmd
}


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
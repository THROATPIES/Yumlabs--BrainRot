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

pub fn get_video_duration(video_path: &str) -> ResultString<f32> {
    let mut cmd = Command::new("python");
    cmd.arg("-c")
        .arg(r#"
from moviepy import *
import sys
try:
    clip = VideoFileClip(sys.argv[1])
    print(clip.duration)
    clip.close()
except Exception as e:
    print(f'Error: {str(e)}', file=sys.stderr)
    sys.exit(1)
        "#)
        .arg(video_path);

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                duration_str.parse::<f32>().map_err(|e| format!("Failed to parse duration: {}", e))
            } else {
                Err(format!("Failed to get video duration: {}", 
                    String::from_utf8_lossy(&output.stderr)))
            }
        }
        Err(e) => Err(format!("Failed to execute duration check: {}", e)),
    }
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
    cmd.arg("-c")
        .arg(r#"
#!C:/Users/THROATPIES/Documents/Development/ipynb_env_3.9/python.exe
import sys
from moviepy import *
import logging
import os

os.environ["PYTHONUTF8"] = "1"
os.environ["PYTHONIOENCODING"] = "utf-8"

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def generate_video(video_clip_path, audio_clip_path, formatted_text, output_video_path, subtitle_fontsize=24, subtitle_color='white'):
    try:
        if not all(os.path.exists(p) for p in [video_clip_path, audio_clip_path]):
            logger.error("Input files not found")
            return False

        video_clip = VideoFileClip(video_clip_path).with_volume_scaled(0.0)
        audio_clip = AudioFileClip(audio_clip_path)
        video_clip = video_clip.with_duration(audio_clip.duration)
        video_clip = video_clip.with_audio(audio_clip)
        video_clip = video_clip.with_volume_scaled(0.8)
        
        subtitle_fontsize = int(subtitle_fontsize)
        dir_font = 'data/inputs/Roboto-Bold.ttf'

        words = [word.replace("\\'", "'") for word in formatted_text.split()]
        audio_clip_duration = audio_clip.duration
        word_duration = audio_clip_duration / len(words)
        
        subtitle_clips = []
        current_time = 0
        for word in words:
            subtitle_clip = (TextClip(text=word,
                                    font=dir_font,
                                    font_size=subtitle_fontsize,
                                    color=subtitle_color,
                                    method='caption',
                                    size=video_clip.size)
                            .with_position(('center', 'bottom'))
                            .with_start(current_time)
                            .with_duration(word_duration))
            subtitle_clips.append(subtitle_clip)
            current_time += word_duration
        
        final_clip = CompositeVideoClip([video_clip] + subtitle_clips)
        final_clip.write_videofile(output_video_path, codec='libx264', fps=24, threads=8, preset='ultrafast', remove_temp=True)
        return True
    except Exception as e:
        print(f'Error: {str(e)}', file=sys.stderr)
        return False

success = generate_video(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], 
                        int(sys.argv[5]) if len(sys.argv) > 5 else 24,
                        sys.argv[6] if len(sys.argv) > 6 else 'white')
sys.exit(0 if success else 1)
        "#)
        .arg(video_clip_path)
        .arg(audio_clip_path)
        .arg(formatted_text)
        .arg(output_video_path);

    if let Some(size) = subtitle_fontsize {
        cmd.arg(size.to_string());
        if let Some(color) = subtitle_color {
            cmd.arg(color);
        }
    }

    execute_command(&mut cmd)
}

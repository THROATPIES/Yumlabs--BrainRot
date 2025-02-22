use std::path::PathBuf;
use std::process::Command;

use crate::constants;

pub struct SplitResult {
    pub video_paths: Vec<PathBuf>,
}

pub fn split_media(
    video_path: &str,
    output_dir: &str,
) -> Result<SplitResult, Box<dyn std::error::Error>> {
    let mut command = Command::new("python");
    command
        .arg("src/media_splitter.py")
        .arg(video_path)
        .arg(output_dir)
        .arg(constants::MAX_VIDEO_DURATION.to_string());

    let output = command.output()?;
    
    if !output.status.success() {
        return Err(format!(
            "Media splitting failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_split_output(&output_str, output_dir)
}

fn parse_split_output(output: &str, output_dir: &str) -> Result<SplitResult, Box<dyn std::error::Error>> {
    let mut video_paths = Vec::new();
    
    for line in output.lines() {
        if line.starts_with("VIDEO:") {
            video_paths.push(PathBuf::from(format!("{}/{}", output_dir, &line[6..])));
        }
    }

    Ok(SplitResult {
        video_paths,
    })
}

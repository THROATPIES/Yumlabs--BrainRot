use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum UploadError {
    CommandFailed(String),
    NoErrorMessage,
    IoError(String),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "Upload failed: {msg}"),
            Self::NoErrorMessage => write!(f, "Upload failed with no error message"),
            Self::IoError(msg) => write!(f, "IO Error: {msg}"),
        }
    }
}

impl std::error::Error for UploadError {}

pub fn handle_upload(
    file_path: &str,
    title: &str,
    description: &str,
    keywords: &str,
    category: &str,
    privacy_status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let child = create_upload_process(
        file_path,
        title,
        description,
        keywords,
        category,
        privacy_status,
    )?;

    process_upload(child)
}

fn create_upload_process(
    file_path: &str,
    title: &str,
    description: &str,
    keywords: &str,
    category: &str,
    privacy_status: &str,
) -> Result<std::process::Child, UploadError> {
    Command::new("python")
        .arg("src/upload_handler.py")
        .arg("--file")
        .arg(file_path)
        .arg("--title")
        .arg(title)
        .arg("--description")
        .arg(description)
        .arg("--keywords")
        .arg(keywords)
        .arg("--category")
        .arg(category)
        .arg("--privacyStatus")
        .arg(privacy_status)
        .arg("--playlistId")
        .arg(crate::constants::YOUTUBE_PLAYLIST_ID)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| UploadError::IoError(e.to_string()))
}

fn process_upload(mut child: std::process::Child) -> Result<(), Box<dyn std::error::Error>> {
    handle_stdout(&mut child);
    handle_process_completion(child)
}

fn handle_stdout(child: &mut std::process::Child) {
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(content) => log_output(&content),
                Err(e) => eprintln!("Error reading stdout: {e}"),
            }
        }
    }
}

fn log_output(line: &str) {
    if line.contains("error") {
        eprintln!("Upload error: {line}");
    } else {
        println!("Upload progress: {line}");
    }
}

fn handle_process_completion(
    mut child: std::process::Child,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = child.wait()?;

    if !status.success() {
        return handle_error(child);
    }

    println!("Upload completed successfully");
    Ok(())
}

fn handle_error(child: std::process::Child) -> Result<(), Box<dyn std::error::Error>> {
    match child.stderr {
        Some(stderr) => {
            let reader = BufReader::new(stderr);
            let error_message = reader
                .lines()
                .filter_map(Result::ok)
                .collect::<Vec<String>>()
                .join("\n");
            Err(UploadError::CommandFailed(error_message).into())
        }
        None => Err(UploadError::NoErrorMessage.into()),
    }
}

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
            UploadError::CommandFailed(msg) => write!(f, "Upload failed: {}", msg),
            UploadError::NoErrorMessage => write!(f, "Upload failed with no error message"),
            UploadError::IoError(msg) => write!(f, "IO Error: {}", msg),
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
    let mut command = Command::new("python");
    command.arg("src/upload_handler.py")
        .arg("--file").arg(file_path)
        .arg("--title").arg(title)
        .arg("--description").arg(description)
        .arg("--keywords").arg(keywords)
        .arg("--category").arg(category)
        .arg("--privacyStatus").arg(privacy_status)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| UploadError::IoError(e.to_string()))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        reader.lines().for_each(|line| {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Error reading stdout: {}", e),
            }
        });
    }

    let status = child.wait()?;
    if !status.success() {
        return match child.stderr {
            Some(stderr) => {
                let reader = BufReader::new(stderr);
                let error_message: String = reader.lines()
                    .filter_map(Result::ok)
                    .collect::<Vec<String>>()
                    .join("\n");
                Err(UploadError::CommandFailed(error_message).into())
            }
            None => Err(UploadError::NoErrorMessage.into()),
        };
    }

    println!("Upload completed successfully");
    Ok(())
}

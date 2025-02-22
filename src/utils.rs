use reqwest;
use serde_json::json;
use std::fs;

use crate::constants;

#[derive(Debug)]
pub struct NotificationError(String);

impl std::error::Error for NotificationError {}
impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Notification error: {}", self.0)
    }
}

pub async fn notify(message: &str, sound: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let data = json!({
        "message": message,
        "sound": sound,
    });

    let resp = client
        .post(constants::NOTIFICATION_URL)
        .json(&data)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(Box::new(NotificationError(format!(
            "Failed to send notification: {:?}",
            resp
        ))));
    }

    Ok(())
}

pub async fn clear_output_folder(folder_path: &str) -> Result<(), std::io::Error> {
    let entries = fs::read_dir(folder_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

pub fn get_current_episode() -> Result<u32, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(constants::EPISODE_FILE_PATH)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    json["episode"]
        .as_u64()
        .ok_or_else(|| "Invalid episode number format".into())
        .map(|n| n as u32)
}

pub fn increment_episode() -> Result<(), Box<dyn std::error::Error>> {
    let current = get_current_episode()?;
    let new_content = json!({ "episode": current + 1 });

    fs::write(
        constants::EPISODE_FILE_PATH,
        serde_json::to_string_pretty(&new_content)?,
    )?;

    Ok(())
}

pub fn sanitize_title(title: &str) -> String {
    let sanitized = title
        .trim()
        .replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), " ")
        .replace("  ", " ")
        .trim()
        .to_string();

    if sanitized.len() > constants::MAX_TITLE_LENGTH {
        return sanitized[..constants::MAX_TITLE_LENGTH].trim().to_string();
    }

    sanitized
}

use std::fs;

use reqwest;
use serde_json::json;

pub async fn notify(message: &str, sound: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8080/notify";

    let data = json!({
        "message": message,
        "sound": sound,
    });

    let resp = client.post(url)
        .json(&data)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Failed to send notification: {:?}", resp);
    } 

    Ok(())
}



pub async fn clear_output_folder(folder_path: &str) -> Result<(), std::io::Error> {
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}


pub fn get_current_episode() -> Result<u32, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("data/current_episode.json")?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    Ok(json["episode"].as_u64().unwrap() as u32)
}

pub fn increment_episode() -> Result<(), Box<dyn std::error::Error>> {
    let current = get_current_episode()?;
    let new_content = json!({
        "episode": current + 1
    });
    fs::write(
        "data/current_episode.json",
        serde_json::to_string_pretty(&new_content)?
    )?;
    Ok(())
}

pub fn sanitize_title(title: &str) -> String {
    let sanitized = title
        .trim()
        .replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), " ")  // Replace non-ASCII/non-alphanumeric chars with space
        .replace("  ", " ")  // Remove double spaces
        .trim()
        .to_string();

    // Ensure the title doesn't exceed YouTube's limits (100 characters is a safe limit)
    if sanitized.len() > 100 {
        sanitized[..100].trim().to_string()
    } else {
        sanitized
    }
}
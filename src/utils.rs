use std::fs;

use reqwest;
use serde_json::json;

pub async fn notify(message: &str, sound: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8080/notify"; // Assuming your server is running locally on port 8080

    let data = json!({
        "message": message,
        "sound": sound,
    });

    let resp = client.post(url)
        .json(&data)
        .send()
        .await?;

    if resp.status().is_success() {
        println!("Notification sent successfully!");
    } else {
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
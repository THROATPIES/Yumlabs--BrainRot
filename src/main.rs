use std::fs;

mod confession;
mod ollama;
mod tts;
mod upload;
mod video;
mod constants;

#[derive(Debug)]
struct VideoMetadata {
    title: String,
    description: String,
    keywords: Vec<String>,
}

async fn clear_output_folder(folder_path: &str) -> Result<(), std::io::Error> {
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

async fn generate_metadata(formatted_text: &str) -> Result<VideoMetadata, Box<dyn std::error::Error>> {
    let movie_title = ollama::generate_title(formatted_text).await?;
    let description = ollama::generate_description(formatted_text).await?;
    let hashtags: Vec<String> = description
        .split_whitespace()
        .filter(|word| word.starts_with('#'))
        .map(|tag| tag[1..].to_string())
        .collect();

    Ok(VideoMetadata {
        title: movie_title,
        description,
        keywords: hashtags,
    })
}

async fn process_video(formatted_text: &str, metadata: &VideoMetadata) -> Result<(), Box<dyn std::error::Error>> {
    video::generate_video(
        constants::VIDEO_INPUT_PATH,
        constants::AUDIO_OUTPUT_PATH,
        formatted_text,
        constants::VIDEO_OUTPUT_PATH,
        constants::VIDEO_FONT_SIZE,
        constants::VIDEO_BG_COLOR,
    )?;

    let formatted_title = format!("Reddit Confessions #{} | {}", constants::CURRENT_EPISODE, metadata.title);
    let keywords_joined = metadata.keywords.join(",");
    upload::handle_upload(
        constants::VIDEO_OUTPUT_PATH,
        &formatted_title,
        &metadata.description,
        &keywords_joined,
        constants::UPLOAD_CATEGORY,
        constants::UPLOAD_PRIVACY,
    )?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    clear_output_folder(constants::OUTPUTS_FOLDER).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("Gathering Data ...");

    let confession_result = confession::read_random_valid_confession()?;
    println!("{:?}", confession_result);

    let formatted_confession = format!("{} {}", confession_result.title, confession_result.selftext);

    let metadata = generate_metadata(&formatted_confession).await?;
    println!("{} \n {} \n {:?}", metadata.title, metadata.description, metadata.keywords);

    tts::generate_tts(
        &formatted_confession,
        constants::AUDIO_OUTPUT_PATH,
        constants::AUDIO_VOICE,
        constants::AUDIO_MODEL,
    )?;

    process_video(&formatted_confession, &metadata).await?;

    Ok(())
}

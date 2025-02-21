
mod confession;
mod constants;
mod ollama;
mod tts;
mod upload;
mod utils;
mod video;
mod splitter;

#[derive(Debug)]
struct VideoMetadata {
    title: String,
    description: String,
    keywords: Vec<String>,
}

async fn generate_metadata(
    formatted_text: &str,
) -> Result<VideoMetadata, Box<dyn std::error::Error>> {
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

async fn process_video(
    formatted_text: &str,
    metadata: &VideoMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    video::generate_video(
        constants::VIDEO_INPUT_PATH,
        constants::AUDIO_OUTPUT_PATH,
        formatted_text,
        constants::VIDEO_OUTPUT_PATH,
        constants::VIDEO_FONT_SIZE,
        constants::VIDEO_BG_COLOR,
    )?;

    let formatted_title = format!(
        "Reddit Confessions #{} | {}",
        constants::CURRENT_EPISODE,
        metadata.title
    );
    let keywords_joined = metadata.keywords.join(",");
    upload::handle_upload(
        constants::VIDEO_OUTPUT_PATH,
        &formatted_title,
        &metadata.description,
        &keywords_joined,
        constants::UPLOAD_CATEGORY,
        constants::UPLOAD_PRIVACY,
    )?;

    // // Check if we need to split the video
    // let split_result = splitter::split_media(
    //     constants::AUDIO_OUTPUT_PATH,
    //     constants::VIDEO_OUTPUT_PATH,
    //     constants::OUTPUTS_FOLDER,
    // )?;

    // // Upload each part
    // for (i, video_path) in split_result.video_paths.iter().enumerate() {
    //     let part_number = i + 1;
    //     let formatted_title = format!(
    //         "Reddit Confessions #{} | {} (Part {}/{})",
    //         constants::CURRENT_EPISODE,
    //         metadata.title,
    //         part_number,
    //         split_result.video_paths.len()
    //     );

    //     let keywords_joined = metadata.keywords.join(",");

    //     println!("{:?}", keywords_joined);
        
    //     upload::handle_upload(
    //         video_path.to_str().unwrap(),
    //         &formatted_title,
    //         &metadata.description,
    //         &keywords_joined,
    //         constants::UPLOAD_CATEGORY,
    //         constants::UPLOAD_PRIVACY,
    //     )?;

    //     // Wait a bit between uploads
    //     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    // }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::clear_output_folder(constants::OUTPUTS_FOLDER).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let _ = utils::notify("Gathering Data ...", "data/sounds/Popup.wav").await;

    let confession_result = confession::read_random_valid_confession()?;
    println!("{:?}", confession_result);

    let formatted_confession =
        format!("{} {}", confession_result.title, confession_result.selftext);

    let metadata = generate_metadata(&formatted_confession).await?;
    println!(
        "{} \n {} \n {:?}",
        metadata.title, metadata.description, metadata.keywords
    );

    tts::generate_tts(
        &formatted_confession,
        constants::AUDIO_OUTPUT_PATH,
        constants::AUDIO_VOICE,
        constants::AUDIO_MODEL,
    )?;

    let _ = utils::notify("Audio Created !!!", "data/sounds/TaskDone.wav").await;

    process_video(&formatted_confession, &metadata).await?;

    let _ = utils::notify("Video Created & Uploaded !!!", "data/sounds/TaskDone.wav").await;

    Ok(())
}

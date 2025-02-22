mod confession;
mod constants;
mod ollama;
mod splitter;
mod tts;
mod upload;
mod utils;
mod video;

use std::time::Duration;

use confession::Confession;

#[derive(Debug)]
struct VideoMetadata {
    title: String,
    description: String,
    keywords: Vec<String>,
}

impl VideoMetadata {
    fn format_title(&self, episode: u32, is_part: Option<(usize, usize)>) -> String {
        match is_part {
            Some((part, total)) => format!(
                "Reddit Confessions #{} | {} (Part {}/{})",
                episode, self.title, part, total
            ),
            None => format!("Reddit Confessions #{} | {} | #shorts", episode, self.title),
        }
    }

    fn get_keywords_string(&self) -> String {
        self.keywords.join(",")
    }
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

async fn generate_base_video(formatted_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    video::execute_python_video_generator(
        constants::VIDEO_INPUT_PATH,
        constants::AUDIO_OUTPUT_PATH,
        formatted_text,
        constants::VIDEO_OUTPUT_PATH,
        constants::VIDEO_FONT_SIZE,
        constants::VIDEO_BG_COLOR,
    )?;

    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(())
}

async fn upload_video(
    video_path: &str,
    metadata: &VideoMetadata,
    episode: u32,
    is_part: Option<(usize, usize)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if constants::IS_DEBUGGING {
        return Ok(());
    }

    let formatted_title = metadata.format_title(episode, is_part);
    let keywords_joined = metadata.get_keywords_string();

    upload::handle_upload(
        video_path,
        &formatted_title,
        &metadata.description,
        &keywords_joined,
        constants::UPLOAD_CATEGORY,
        constants::UPLOAD_PRIVACY,
    )?;

    Ok(())
}

async fn process_short_video(
    formatted_text: &str,
    metadata: &VideoMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    generate_base_video(formatted_text).await?;

    let episode = utils::get_current_episode()?;
    upload_video(constants::VIDEO_OUTPUT_PATH, metadata, episode, None).await?;
    if !constants::IS_DEBUGGING {
        utils::increment_episode()?;
    }

    Ok(())
}

async fn process_long_video(
    metadata: &VideoMetadata,
    formatted_confession: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    generate_base_video(formatted_confession).await?;

    let split_result =
        splitter::split_media(constants::VIDEO_OUTPUT_PATH, constants::OUTPUTS_FOLDER)?;

    let total_parts = split_result.video_paths.len();
    let episode = utils::get_current_episode()?;

    for (i, video_path) in split_result.video_paths.iter().enumerate() {
        let part_number = i + 1;

        if !video_path.to_str().unwrap().contains("original_") {
            upload_video(
                video_path.to_str().unwrap(),
                metadata,
                episode,
                Some((part_number, total_parts)),
            )
            .await?;

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    if !constants::IS_DEBUGGING {
        utils::increment_episode()?;
    }

    Ok(())
}

async fn notify_with_sound(
    message: &str,
    sound_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    utils::notify(message, sound_path).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(())
}

async fn get_valid_confession_and_metadata(
) -> Result<(Confession, VideoMetadata), Box<dyn std::error::Error>> {
    for attempt in 0..constants::MAX_RETRIES {
        let confession_result = confession::read_random_valid_confession()?;
        let formatted_confession =
            format!("{} {}", confession_result.title, confession_result.selftext);

        match generate_metadata(&formatted_confession).await {
            Ok(metadata) => {
                if !metadata
                    .title
                    .to_lowercase()
                    .contains("cannot create content")
                    && !metadata.title.to_lowercase().contains("i cannot")
                    && !metadata.title.to_lowercase().contains("unable to process")
                {
                    // Generate TTS and check duration
                    tts::generate_tts(
                        &formatted_confession,
                        constants::AUDIO_OUTPUT_PATH,
                        constants::AUDIO_VOICE,
                        constants::AUDIO_MODEL,
                    )?;

                    let video_duration =
                        video::get_duration_from_audio(constants::AUDIO_OUTPUT_PATH)?;

                    if video_duration >= constants::MINIMUM_VIDEO_DURATION {
                        println!(
                            "Valid confession found on attempt {} with duration {:.2}s",
                            attempt + 1,
                            video_duration
                        );
                        return Ok((confession_result, metadata));
                    } else {
                        println!(
                            "Confession duration {:.2}s too short (minimum {:.2}s). Retrying...",
                            video_duration,
                            constants::MINIMUM_VIDEO_DURATION
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Metadata generation failed on attempt {}: {}",
                    attempt + 1,
                    e
                );
            }
        }

        if attempt < constants::MAX_RETRIES - 1 {
            println!(
                "Retrying with new confession... ({}/{})",
                attempt + 1,
                constants::MAX_RETRIES
            );
        }
    }

    Err("Failed to find acceptable confession after maximum retries".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::clear_output_folder(constants::OUTPUTS_FOLDER).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    notify_with_sound("Gathering Data ...", "data/sounds/Ani_Alert.wav").await?;

    let (confession_result, metadata) = get_valid_confession_and_metadata().await?;
    let formatted_confession =
        format!("{} {}", confession_result.title, confession_result.selftext);

    println!(
        "Metadata: {} \n {} \n {:?}",
        metadata.title, metadata.description, metadata.keywords
    );

    notify_with_sound("Audio Created !!!", "data/sounds/Ani_Success.wav").await?;

    let video_duration = video::get_duration_from_audio(constants::AUDIO_OUTPUT_PATH)?;
    println!("Video Duration: {} seconds", video_duration);

    if video_duration <= constants::MAX_VIDEO_DURATION {
        notify_with_sound("Short Video ...", "data/sounds/Ani_Alert.wav").await?;
        process_short_video(&formatted_confession, &metadata).await?;
    } else {
        notify_with_sound("Long Video ...", "data/sounds/Ani_Alert.wav").await?;
        process_long_video(&metadata, &formatted_confession).await?;
    }

    tokio::time::sleep(Duration::from_secs(2)).await;
    notify_with_sound(
        "Video Created & Uploaded !!!",
        "data/sounds/Ani_Success.wav",
    )
    .await?;

    Ok(())
}

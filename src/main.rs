mod confession;
mod constants;
mod ollama;
mod splitter;
mod tts;
mod upload;
mod utils;
mod video;
// mod video_generator;

use std::time::Duration;
use tokio::task;
use futures::future::join_all;

use confession::Confession;

#[derive(Debug, Clone)]
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

fn estimate_duration_from_text(text: &str) -> f32 {
    // Average speaking rate is about 150 words per minute
    // So each word takes approximately 0.4 seconds
    const SECONDS_PER_WORD: f32 = 0.4;
    
    let word_count = text.split_whitespace().count();
    word_count as f32 * SECONDS_PER_WORD
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
    // video_generator::generate_video_from_args(
    //     constants::VIDEO_INPUT_PATH,
    //         constants::AUDIO_OUTPUT_PATH,
    //         formatted_text,
    //         constants::VIDEO_OUTPUT_PATH,
    //         constants::VIDEO_FONT_SIZE,
    //         constants::VIDEO_BG_COLOR,
    // )?;

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
) -> Result<(), Box<dyn std::error::Error + Send>> {
    generate_base_video(formatted_confession).await.unwrap();

    let split_result = 
        splitter::split_media(constants::VIDEO_OUTPUT_PATH, constants::OUTPUTS_FOLDER).unwrap();

    let total_parts = split_result.video_paths.len();
    let episode = utils::get_current_episode().unwrap();

    // Create a vector to hold all upload tasks
    let mut upload_tasks = Vec::new();

    for (i, video_path) in split_result.video_paths.iter().enumerate() {
        let part_number = i + 1;
        
        if !video_path.to_str().unwrap().contains("original_") {
            // Clone necessary values for the async task
            let video_path = video_path.to_str().unwrap().to_string();
            let metadata = metadata.clone();  // Requires #[derive(Clone)] on VideoMetadata

            // Spawn a new task for each upload
            let upload_task = task::spawn(async move {
                upload_video(
                    &video_path,
                    &metadata,
                    episode,
                    Some((part_number, total_parts)),
                ).await
                .map_err(|e| -> Box<dyn std::error::Error + Send> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Upload failed: {}", e)
                    ))
                })
            });

            upload_tasks.push(upload_task);
        }
    }

    // Wait for all uploads to complete
    join_all(upload_tasks).await
        .into_iter()
        .collect::<Result<Vec<_>, _>>().unwrap()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    if !constants::IS_DEBUGGING {
        utils::increment_episode().unwrap();
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
                    // Estimate duration before generating TTS
                    let estimated_duration = estimate_duration_from_text(&formatted_confession);

                    if estimated_duration >= constants::MINIMUM_VIDEO_DURATION {
                        println!(
                            "Valid confession found on attempt {} with estimated duration {:.2}s",
                            attempt + 1,
                            estimated_duration
                        );
                        
                        // Generate TTS only after we know the estimated duration is acceptable
                        tts::generate_tts(
                            &formatted_confession,
                            constants::AUDIO_OUTPUT_PATH,
                            constants::AUDIO_VOICE,
                            constants::AUDIO_MODEL,
                        )?;

                        return Ok((confession_result, metadata));
                    } else {
                        println!(
                            "Estimated confession duration {:.2}s too short (minimum {:.2}s). Retrying...",
                            estimated_duration,
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
        process_long_video(&metadata, &formatted_confession).await.unwrap();
    }

    tokio::time::sleep(Duration::from_secs(2)).await;
    notify_with_sound(
        "Video Created & Uploaded !!!",
        "data/sounds/Ani_Success.wav",
    )
    .await?;

    Ok(())
}

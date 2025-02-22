mod confession;
mod constants;
mod ollama;
mod splitter;
mod tts;
mod upload;
mod utils;
mod video;

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
        utils::get_current_episode()?,
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

    utils::increment_episode()?;
    Ok(())
}

async fn split_and_upload(
    metadata: &VideoMetadata,
    formatted_confession: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // First, check the video duration
    let video_duration = video::get_video_duration(constants::VIDEO_OUTPUT_PATH)?;

    if video_duration <= constants::MAX_VIDEO_DURATION {
        // If video is shorter than max duration, just upload it directly
        let formatted_title = format!(
            "Reddit Confessions #{} | {}",
            utils::get_current_episode()?,
            metadata.title
        );
        let keywords_joined = metadata.keywords.join(",");
        if !constants::IS_DEBUGGING {
            upload::handle_upload(
                constants::VIDEO_OUTPUT_PATH,
                &formatted_title,
                &metadata.description,
                &keywords_joined,
                constants::UPLOAD_CATEGORY,
                constants::UPLOAD_PRIVACY,
            )?;
        }

        return Ok(());
    }

    // Only split if video is longer than MAX_VIDEO_DURATION
    let split_result = splitter::split_media(
        constants::AUDIO_OUTPUT_PATH,
        constants::VIDEO_OUTPUT_PATH,
        constants::OUTPUTS_FOLDER,
    )?;

    // Process both audio and video files
    for (i, (audio_path, video_path)) in split_result
        .audio_paths
        .iter()
        .zip(split_result.video_paths.iter())
        .enumerate()
    {
        let part_number = i + 1;
        let formatted_title = format!(
            "Reddit Confessions #{} | {} (Part {}/{})",
            utils::get_current_episode()?,
            metadata.title,
            part_number,
            split_result.video_paths.len()
        );

        let combined_video_path = format!(
            "{}/combined_part_{}.mp4",
            constants::OUTPUTS_FOLDER,
            part_number
        );

        // Generate new video with the split audio and video
        video::execute_python_video_generator(
            video_path.to_str().unwrap(),
            audio_path.to_str().unwrap(),
            &formatted_confession,
            &combined_video_path,
            constants::VIDEO_FONT_SIZE,
            constants::VIDEO_BG_COLOR,
        )?;

        let keywords_joined = metadata.keywords.join(",");
        if !constants::IS_DEBUGGING {
            upload::handle_upload(
                &combined_video_path,
                &formatted_title,
                &metadata.description,
                &keywords_joined,
                constants::UPLOAD_CATEGORY,
                constants::UPLOAD_PRIVACY,
            )?;
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::clear_output_folder(constants::OUTPUTS_FOLDER).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let _ = utils::notify("Gathering Data ...", "data/sounds/Popup.wav").await;

    let confession_result = confession::read_random_valid_confession()?;
    println!("Confession Result: {:?}", confession_result);

    let formatted_confession =
        format!("{} {}", confession_result.title, confession_result.selftext);

    let metadata = generate_metadata(&formatted_confession).await?;
    println!(
        "Metadata Shit: {} \n {} \n {:?}",
        metadata.title, metadata.description, metadata.keywords
    );

    tts::generate_tts(
        &formatted_confession,
        constants::AUDIO_OUTPUT_PATH,
        constants::AUDIO_VOICE,
        constants::AUDIO_MODEL,
    )?;

    let _ = utils::notify("Audio Created !!!", "data/sounds/TaskDone.wav").await;

    video::execute_python_video_generator(
        constants::VIDEO_INPUT_PATH,
        constants::AUDIO_OUTPUT_PATH,
        &formatted_confession,
        constants::VIDEO_OUTPUT_PATH,
        constants::VIDEO_FONT_SIZE,
        constants::VIDEO_BG_COLOR,
    )?;

    if constants::IS_DEBUGGING {
        split_and_upload(&metadata, &formatted_confession).await?;
        return Ok(());
    } else {
        process_video(&formatted_confession, &metadata).await?;
    }

    let _ = utils::notify("Video Created & Uploaded !!!", "data/sounds/TaskDone.wav").await;

    Ok(())
}

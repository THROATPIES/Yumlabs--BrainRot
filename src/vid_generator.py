#!C:/Users/THROATPIES/Documents/Development/ipynb_env_3.9/python.exe
import sys
from moviepy import *
import logging
import os

os.environ["PYTHONUTF8"] = "1"  # Enable UTF-8 mode in Python 3.7+
os.environ["PYTHONIOENCODING"] = "utf-8"

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def generate_video(
    video_clip_path: str,
    audio_clip_path: str,
    formatted_text: str,
    output_video_path: str,
    subtitle_fontsize: int = 24,
    subtitle_color: str = 'white'
) :
    """
    Generates a video by integrating audio and overlaying subtitles.
    Returns True on success, False on failure.
    """
    try:
        if not all(os.path.exists(p) for p in [video_clip_path, audio_clip_path]):
            logger.error("Input files not found")
            return False

        video_clip = VideoFileClip(video_clip_path).with_volume_scaled(0.0)
        audio_clip = AudioFileClip(audio_clip_path)

        video_clip = video_clip.with_duration(audio_clip.duration)
        
        video_clip = video_clip.with_audio(audio_clip)
        video_clip = video_clip.with_volume_scaled(0.8)
        
        subtitle_fontsize = int(subtitle_fontsize)
        dir_font = "data/Roboto-Bold.ttf"

        words = [word.replace("\\'", "'") for word in formatted_text.split()]
        
        audio_clip_duration = audio_clip.duration
        word_duration = audio_clip_duration / len(words)
        
        subtitle_clips = []
        current_time = 0
        for word in words:
            subtitle_clip = (TextClip(text=word,
                                    font=dir_font,
                                    font_size=subtitle_fontsize,
                                    color=subtitle_color,
                                    method='caption',
                                    size=video_clip.size)
                            .with_position(('center', 'bottom'))
                            .with_start(current_time)
                            .with_duration(word_duration))
            subtitle_clips.append(subtitle_clip)
            current_time += word_duration
        
        final_clip = CompositeVideoClip([video_clip] + subtitle_clips)
        
        final_clip.write_videofile(
            output_video_path,
            codec='libx264',
            fps=24,
            threads=8,
            preset='ultrafast',
            remove_temp=True,
            temp_audiofile_path='data/output',
            logger='bar'  
        )
        
        return True

    except Exception as e:
        logger.error(f"Error generating video: {str(e)}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: video_clip_path audio_clip_path formatted_text output_video_path [font] [fontsize] [color]")
        sys.exit(1)
    
    success = generate_video(*sys.argv[1:])
    sys.exit(0 if success else 1)

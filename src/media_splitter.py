import sys
from moviepy import *
import math
import os

def split_media(audio_path, video_path, output_dir, max_duration):
    # Load the audio and video
    audio = AudioFileClip(audio_path)
    video = VideoFileClip(video_path)
    
    total_duration = audio.duration
    num_parts = math.ceil(total_duration / max_duration)
    
    audio_paths = []
    video_paths = []
    
    for i in range(num_parts):
        start_time = i * max_duration
        end_time = min((i + 1) * max_duration, total_duration)
        
        # Split audio
        audio_part = audio.subclipped(start_time, end_time)
        audio_output = f"part_{i+1}_audio.wav"
        audio_full_path = os.path.join(output_dir, audio_output)
        audio_part.write_audiofile(audio_full_path)
        print(f"AUDIO:{audio_output}")
        audio_paths.append(audio_output)
        
        # Split video
        video_part = video.subclipped(start_time, end_time)
        video_output = f"part_{i+1}_video.mp4"
        video_full_path = os.path.join(output_dir, video_output)
        video_part.write_videofile(video_full_path, audio=False)
        print(f"VIDEO:{video_output}")
        video_paths.append(video_output)
        
    audio.close()
    video.close()
    
    return audio_paths, video_paths

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: audio_path video_path output_dir max_duration")
        sys.exit(1)
        
    audio_path = sys.argv[1]
    video_path = sys.argv[2]
    output_dir = sys.argv[3]
    max_duration = float(sys.argv[4])
    
    split_media(audio_path, video_path, output_dir, max_duration)
import sys
import math
import os
import shutil
import subprocess

def split_media(video_path, output_dir, max_duration):
    # First, make a copy of the original video
    original_filename = os.path.basename(video_path)
    backup_path = os.path.join(output_dir, f"original_{original_filename}")
    shutil.copy2(video_path, backup_path)
    
    # Get video duration using ffprobe
    probe = subprocess.run([
        'ffprobe', 
        '-v', 'error',
        '-show_entries', 'format=duration',
        '-of', 'default=noprint_wrappers=1:nokey=1',
        video_path
    ], capture_output=True, text=True)
    
    total_duration = float(probe.stdout)
    num_parts = math.ceil(total_duration / max_duration)
    video_paths = []
    
    for i in range(num_parts):
        start_time = i * max_duration
        end_time = min((i + 1) * max_duration, total_duration)
        
        video_output = f"part_{i+1}_video.mp4"
        video_full_path = os.path.join(output_dir, video_output)
        
        # Use FFmpeg to split without re-encoding
        subprocess.run([
            'ffmpeg',
            '-i', video_path,
            '-ss', str(start_time),
            '-t', str(end_time - start_time),
            '-c', 'copy',  # Copy without re-encoding
            '-avoid_negative_ts', '1',
            video_full_path
        ], check=True)
        
        print(f"VIDEO:{video_output}")
        video_paths.append(video_output)
    
    return video_paths

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: video_path output_dir max_duration")
        sys.exit(1)
        
    video_path = sys.argv[1]
    output_dir = sys.argv[2]
    max_duration = float(sys.argv[3])
    
    split_media(video_path, output_dir, max_duration)


#!C:/Users/THROATPIES/Documents/Development/ipynb_env_3.9/python.exe

import sys
from kokoro import KPipeline
import torch
import soundfile as sf

def generate_tts(text, output_audio_path, voice='af_bella', lang_code='a'):
    """Generates TTS audio using Kokoro pipeline."""
    pipeline = KPipeline(lang_code=lang_code) 

    generator = pipeline(
        text, voice=voice, 
        speed=1, split_pattern=r'\n+'
    )

    all_audio = []
    for i, (gs, ps, audio) in enumerate(generator):
        all_audio.append(audio)

    if all_audio:
        final_audio = torch.cat(all_audio, dim=0)

        sf.write(output_audio_path, final_audio, 24000)
        print(f"TTS audio generated successfully at: {output_audio_path}")
    else:
        print("No audio generated.")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python tts_generator.py <text_to_speak> <output_audio_path> [voice] [lang_code]")
        print("  - <text_to_speak>: The text to convert to speech.")
        print("  - <output_audio_path>: Path to save the output audio file (.wav).")
        print("  - [voice] (optional): Voice to use (default: af_bella).")
        print("  - [lang_code] (optional): Language code (default: a).")
        sys.exit(1)

    text_to_speak = sys.argv[1]
    output_audio_path = sys.argv[2]
    voice = sys.argv[3] if len(sys.argv) > 3 else 'af_bella' 
    lang_code = sys.argv[4] if len(sys.argv) > 4 else 'a'   

    try:
        generate_tts(text_to_speak, output_audio_path, voice, lang_code)
    except Exception as e:
        print(f"TTS generation failed: {e}")
        sys.exit(1)
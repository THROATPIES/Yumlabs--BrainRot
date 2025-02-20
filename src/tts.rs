use std::path::Path;
use std::process::Command;

pub fn generate_tts(
    text: &str,
    output_path: &str,
    voice: Option<&str>,
    lang_code: Option<&str>,
) -> Result<(), String> {
    let script_path = Path::new("src/tts_generator.py");

    let mut cmd = Command::new("python");
    cmd.arg(script_path).arg(text).arg(output_path);

    if let Some(v) = voice {
        cmd.arg(v);
        if let Some(l) = lang_code {
            cmd.arg(l);
        }
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("TTS generation failed: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute TTS script: {}", e)),
    }
}

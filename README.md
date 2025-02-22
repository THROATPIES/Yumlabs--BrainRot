<div align="center">

# 🎬 Reddit Confessions Video Generator

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3670A0?style=for-the-badge&logo=python&logoColor=ffdd54)](https://www.python.org/)
[![YouTube](https://img.shields.io/badge/YouTube-%23FF0000.svg?style=for-the-badge&logo=YouTube&logoColor=white)](https://www.youtube.com/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)

Automagically generate and upload Reddit confession videos with AI voiceovers, synchronized captions, and dynamic titles. Built with Rust 🦀 and Python 🐍.

[Key Features](#-key-features) •
[Installation](#-installation) •
[Usage](#-usage) •
[Documentation](#-documentation) •
[Contributing](#-contributing)

<!-- <img src="docs/demo.gif" alt="Demo" width="600"/> -->

</div>

## ✨ Key Features

- 🤖 **AI-Powered Generation**
  - Text-to-speech using Kokoro TTS
  - Dynamic titles and descriptions via Ollama
  - Smart text cleanup and formatting

- 🎥 **Professional Video Production**
  - Word-by-word caption synchronization
  - High-contrast text with outlines
  - Automatic video splitting for long content
  - Background music integration

- 🚀 **Automation**
  - YouTube upload automation
  - Hashtag generation
  - Progress notifications
  - Error handling and retries

## 🛠 Installation

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Python dependencies
pip install moviepy kokoro torch soundfile imageio-ffmpeg

# External dependencies
brew install ffmpeg ollama  # or your system's package manager
```

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/reddit-confessions-generator
cd reddit-confessions-generator

# Build the project
cargo build --release

# Set up project structure
mkdir -p data/{inputs,output,sounds}

# Install dependencies
cargo build
pip install -r requirements.txt
```

## 📦 Project Structure

```
reddit-confessions-generator/
├── 🦀 src/
│   ├── main.rs          # Core application logic
│   ├── ollama.rs        # AI text generation
│   ├── confession.rs    # Data handling
│   └── upload.rs        # YouTube integration
├── 🐍 python/
│   ├── tts_generator.py # Text-to-speech
│   └── vid_generator.py # Video processing
└── 📂 data/
    ├── inputs/          # Source files
    ├── output/          # Generated content
    └── sounds/          # Notification sounds
```

## 🚀 Usage

1. **Configure Your Environment**
   ```bash
   # Place required files
   cp your-background.mp4 data/inputs/input.mp4
   cp Roboto-Bold.ttf data/inputs/
   ```

2. **Run the Generator**
   ```bash
   cargo run --release
   ```

3. **Watch the Magic Happen**
   - Confession selection ✨
   - AI title generation 🤖
   - TTS processing 🎙
   - Video creation 🎬
   - YouTube upload 🚀

## 📚 Documentation

<details>
<summary>Configuration Options</summary>

```rust
// Constants available in src/constants.rs
pub const MAX_VIDEO_DURATION: f64 = 60.0;
pub const VIDEO_FONT_SIZE: i32 = 48;
pub const AUDIO_VOICE: &str = "af_bella";
```
</details>

<details>
<summary>YouTube Upload Setup</summary>

1. Create project in Google Cloud Console
2. Enable YouTube Data API
3. Configure OAuth 2.0
4. Place credentials in `docs/sec.json`
</details>

## 🤝 Contributing

Contributions are what make the open source community amazing! Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

Distributed under the MIT License. See `LICENSE` for more information.

## 🙏 Acknowledgments

- [Ollama](https://github.com/ollama/ollama) for AI text generation
- [Kokoro](https://github.com/kokoro) for TTS capabilities
- [MoviePy](https://github.com/Zulko/moviepy) for video processing

---

<div align="center">

Made with ❤️ by [THROATPIES](https://github.com/THROATPIES)

⭐ Star us on GitHub — it motivates us a lot!

</div>

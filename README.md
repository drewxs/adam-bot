# Adam (Discord Bot)

Models (OpenAI): gpt-3.5-turbo, whisper-1

## Features

- Messaging
  - Reply detection
  - Rate limiting
- Comprehensive logging
- Music
  - YouTube search
  - Queue controls
- Voice
  - Live transcriptions
  - Transcription-based replies
  - Text to speech
  - Music controls

## Development

#### Requirements

For local development:

- rust
- ffmpeg
- opus
- yt-dlp

OR

For Dockerized development:

- docker
- docker-compose

Create a `.env` file from `.env.example`, then tweak `src/cfg.rs` to your needs.

Running:

```sh
# Locally
cargo run

# Using Docker
docker-compose up
```

### Fine-tuning

#### Requirements

- `python@^3.11`
- `poetry`

Create a new file `model/<name>.jsonl` and update the path in `model/tune.py`.
Alternatively, update `model/train.jsonl` directly.

To queue up a fine-tuning job on OpenAI:

```sh
cd model
poetry shell
poetry install
poetry run python tune.py
```

---

[License](https://github.com/drewxs/adam-bot/blob/main/LICENSE)

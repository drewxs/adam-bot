# Adam Bot

Discord bot based on a friend.

## Development

#### Requirements

- `rust` or `docker`, `docker-compose`

Create `.env` from `.env.example`, tweak `src/cfg.rs`.

Then either run `cargo run` or `docker-compose up`.

### Fine-tuning

#### Requirements

- `python@^3.11`
- `poetry`

Create a new file `model/<name>.jsonl` then update the path in `model/tune.py`, or update `model/train.jsonl`.

To queue up a fine-tuning job on OpenAI:

```sh
cd model
poetry shell
poetry install
poetry run python tune.py
```

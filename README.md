# cardturner

A Rust CLI that asks a local [Ollama](https://ollama.com) model to bid contract bridge hands using the **Standard American Yellow Card (SAYC)** convention system.

## Quick start

```sh
ollama pull gemma4:26b   # or any model you have locally
ollama serve

cargo run -- \
  --hand "S:KQ4 H:AJ9 D:KT76 C:KQ3" \
  --dealer N \
  --vul None \
  --auction ""
```

Output is a JSON object:

```json
{
  "bid": "1NT",
  "reason": "Balanced 18 HCP — open 1NT (15-17 in SAYC)."
}
```

## CLI

| Flag              | Default                     | Notes                                                |
|-------------------|-----------------------------|------------------------------------------------------|
| `--hand`          | required                    | `S:AKQ4 H:JT9 D:8765 C:K32` — `T` or `1` for ten    |
| `--dealer`        | required                    | `N`, `E`, `S`, `W`                                   |
| `--vul`           | `None`                      | `None`, `NS`, `EW`, `Both`                           |
| `--auction`       | empty                       | Space-separated calls: `P`, `X`, `XX`, `1NT`, `4S`   |
| `--model`         | `gemma4:26b`                | Any Ollama model name                                |
| `--ollama-url`    | `http://localhost:11434`    | Ollama base URL                                      |
| `--system-prompt` | `prompts/sayc.md`           | Path to the system prompt                            |

## Testing

```sh
cargo test                                  # unit + mocked integration
cargo test scenarios -- --ignored           # live-Ollama scenario fixtures
```

See `fixtures/README.md` for how to add scenarios. See `CLAUDE.md` for the layout map and conventions.

# Bidding scenario fixtures

Each file under `scenarios/` is one bidding situation we expect the LLM to handle. They are exercised by `tests/scenarios.rs` against a live Ollama instance:

```
cargo test scenarios -- --ignored
```

(Override target with `OLLAMA_URL` and `OLLAMA_MODEL`.)

## Format

```json
{
  "name": "Open 1NT with balanced 15-17",
  "hand": "S:KQ4 H:AJ9 D:KT76 C:KQ3",
  "dealer": "N",
  "vul": "None",
  "auction": "",
  "expected_bid": "1NT",
  "notes": "Balanced 18 HCP — but 18 is too many for 1NT in SAYC; this is a tuning case."
}
```

- `hand` uses the suit-prefixed format: `S:... H:... D:... C:...`. Use `T` (or `1`) for ten. Voids are `S:` (empty) or `S:-`.
- `dealer` is `N`, `E`, `S`, or `W`.
- `vul` is `None`, `NS`, `EW`, or `Both`.
- `auction` is space-separated calls (`P`, `X`, `XX`, or `1NT`/`4S`/...). Empty means the dealer is on call.
- `expected_bid` is matched case-insensitively against `bid` from the LLM.

## Adding a scenario

1. Drop a new `.json` file in `scenarios/`.
2. Re-run `cargo test scenarios -- --ignored`. The harness picks it up automatically.

When a scenario fails, the failure message includes the LLM's `reason` so you can iterate on `prompts/sayc.md` instead of fighting the test.

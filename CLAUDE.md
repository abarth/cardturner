# cardturner — notes for coding agents

## What this is

A Rust CLI that asks a locally-hosted Ollama LLM to make the next call in a contract bridge auction using the Standard American Yellow Card (SAYC) convention system. One-shot per invocation: pass a hand, dealer, vulnerability, and the auction so far; print a JSON object `{"bid": "...", "reason": "..."}`.

## Layout

```
src/
  lib.rs        — re-exports the public API
  main.rs       — clap CLI; reads system prompt from disk; calls bid()
  hand.rs       — Hand/Suit/Rank, FromStr for "S:AKQ4 H:JT9 D:8765 C:K32"
  auction.rs    — Call/Strain/Seat/Vulnerability/Auction; parses "1S P 2H P"
  ollama.rs     — LlmClient trait + OllamaClient (POST {url}/api/chat, format:json)
  prompt.rs     — build_user_message(hand, auction) — assembles the user message
  bid.rs        — bid(hand, auction, system_prompt, &dyn LlmClient) -> BidResponse
prompts/
  sayc.md       — system prompt; SAYC primer + bid notation + JSON output contract
fixtures/
  scenarios/    — *.json bidding cases for live-Ollama tests
tests/          — unit & integration tests; tests/scenarios.rs is #[ignore]
```

## Running locally

```
ollama pull gemma4:26b      # or any model you prefer
ollama serve                # in another terminal
cargo run -- \
  --hand "S:AKQ4 H:KQJ D:KT9 C:K32" \
  --dealer N --vul None --auction ""
```

Defaults: `--model gemma4:26b`, `--ollama-url http://localhost:11434`, `--system-prompt prompts/sayc.md`, `--vul None`.

## Testing

- `cargo test` — fast: unit tests + mocked end-to-end. No Ollama required.
- `cargo test scenarios -- --ignored` — runs every `fixtures/scenarios/*.json` against a real Ollama. Override target with `OLLAMA_URL` / `OLLAMA_MODEL`.
- Snapshot tests use `insta`. To accept a deliberate prompt change: `cargo install cargo-insta` and `cargo insta review`.

## Conventions

- **All LLM calls go through the `LlmClient` trait** in `ollama.rs`. Don't call `reqwest` from business logic — that breaks the test mock and locks us to one transport.
- **`Hand` and `Auction` have `FromStr` impls.** Reuse them; do not reparse those strings ad hoc.
- **Iterate on bidding behavior by editing `prompts/sayc.md`,** not by patching code. The file is read at runtime.
- **Add new bidding scenarios as JSON files** under `fixtures/scenarios/`. The harness picks them up automatically.
- **Errors:** library modules use `thiserror`; `main.rs` uses `anyhow` to add context. Keep that split — tests want typed errors.
- **Async runtime:** `tokio` with the multi-thread runtime via `#[tokio::main]`. Tests use `#[tokio::test]`.

## Non-goals (for now)

- Card play, dummy display, declarer-side analysis.
- A REPL — `bid()` is the right seam if we ever want one.
- Bid legality enforcement beyond syntactic validation.
- Persistent state, multi-language support, network deployment.

//! Live-Ollama scenario harness. Each `fixtures/scenarios/*.json` is one case.
//! These tests are `#[ignore]` so they only run with `cargo test -- --ignored`
//! (or `cargo test scenarios -- --ignored`).
//!
//! Set `OLLAMA_URL` and `OLLAMA_MODEL` env vars to override the defaults.

use std::fs;
use std::path::{Path, PathBuf};

use cardturner::{bid, Auction, Hand, OllamaClient, Seat, Vulnerability};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    hand: String,
    dealer: String,
    vul: String,
    #[serde(default)]
    auction: String,
    expected_bid: String,
    #[serde(default)]
    notes: String,
}

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/scenarios")
}

fn read_system_prompt() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("prompts/sayc.md");
    fs::read_to_string(&path).expect("prompts/sayc.md must be readable")
}

async fn run_scenario(path: &Path) {
    let raw = fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let scenario: Scenario = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));

    let hand: Hand = scenario.hand.parse().expect("hand parse");
    let dealer: Seat = scenario.dealer.parse().expect("dealer parse");
    let vul: Vulnerability = scenario.vul.parse().expect("vul parse");
    let auction = Auction::parse(dealer, vul, &scenario.auction).expect("auction parse");

    let url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:26b".into());
    let client = OllamaClient::new(url, model);

    let response = bid(&hand, &auction, &read_system_prompt(), &client)
        .await
        .unwrap_or_else(|e| panic!("[{}] bid call failed: {e}", scenario.name));

    assert_eq!(
        response.bid.trim().to_uppercase(),
        scenario.expected_bid.trim().to_uppercase(),
        "[{}] expected {}, got {} (reason: {}). Notes: {}",
        scenario.name,
        scenario.expected_bid,
        response.bid,
        response.reason,
        scenario.notes,
    );
}

#[tokio::test]
#[ignore]
async fn run_all_scenarios() {
    let dir = scenarios_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();

    assert!(!entries.is_empty(), "no fixture .json files in {}", dir.display());

    for entry in entries {
        run_scenario(&entry.path()).await;
    }
}

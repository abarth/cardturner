use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use cardturner::{bid, Auction, Hand, OllamaClient, Seat, Vulnerability};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "cardturner",
    about = "Ask a local Ollama model to bid a contract bridge hand using SAYC.",
    version
)]
struct Args {
    /// The bidder's hand, e.g. "S:AKQ4 H:JT9 D:8765 C:K32".
    #[arg(long)]
    hand: String,

    /// Dealer seat: N, E, S, or W.
    #[arg(long)]
    dealer: String,

    /// Vulnerability: None, NS, EW, Both. Defaults to None.
    #[arg(long, default_value = "None")]
    vul: String,

    /// Auction so far, space-separated calls (P, X, XX, or like 1NT). May be empty.
    #[arg(long, default_value = "")]
    auction: String,

    /// Ollama model name.
    #[arg(long, default_value = "gemma4:26b")]
    model: String,

    /// Ollama base URL.
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Path to the system prompt (Markdown). Defaults to `prompts/sayc.md`.
    #[arg(long, default_value = "prompts/sayc.md")]
    system_prompt: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    let hand: Hand = args
        .hand
        .parse()
        .with_context(|| format!("parsing --hand {:?}", args.hand))?;
    let dealer: Seat = args
        .dealer
        .parse()
        .with_context(|| format!("parsing --dealer {:?}", args.dealer))?;
    let vul: Vulnerability = args
        .vul
        .parse()
        .with_context(|| format!("parsing --vul {:?}", args.vul))?;
    let auction = Auction::parse(dealer, vul, &args.auction)
        .with_context(|| format!("parsing --auction {:?}", args.auction))?;

    let system_prompt = std::fs::read_to_string(&args.system_prompt)
        .with_context(|| format!("reading system prompt at {}", args.system_prompt.display()))?;

    let client = OllamaClient::new(args.ollama_url, args.model);
    let response = bid(&hand, &auction, &system_prompt, &client)
        .await
        .context("LLM bid request failed")?;

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

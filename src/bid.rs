use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, trace};

use crate::auction::{Auction, Call};
use crate::hand::Hand;
use crate::ollama::{LlmClient, LlmError};
use crate::prompt::build_user_message;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BidResponse {
    /// The call as the LLM expressed it, e.g. "1NT", "Pass", "X".
    pub bid: String,
    /// Short rationale from the LLM.
    pub reason: String,
}

impl BidResponse {
    /// Parse the `bid` field into a `Call`. Returns an error if the LLM produced
    /// something we don't recognize.
    pub fn parsed_call(&self) -> Result<Call, BidError> {
        Call::parse(self.bid.trim()).map_err(|_| BidError::UnrecognizedCall(self.bid.clone()))
    }
}

#[derive(Debug, Error)]
pub enum BidError {
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error("LLM response was not valid JSON: {0}\n--- raw ---\n{1}")]
    BadJson(serde_json::Error, String),
    #[error("LLM returned an unrecognized call: {0}")]
    UnrecognizedCall(String),
}

/// Build the prompt, ask the LLM, and parse the response.
pub async fn bid(
    hand: &Hand,
    auction: &Auction,
    system_prompt: &str,
    client: &dyn LlmClient,
) -> Result<BidResponse, BidError> {
    let user = build_user_message(hand, auction);
    debug!(user_message_chars = user.len(), "built user message");
    trace!(user_message = %user, "user message");
    let raw = client.chat(system_prompt, &user).await?;
    let parsed: BidResponse =
        serde_json::from_str(&raw).map_err(|e| BidError::BadJson(e, raw.clone()))?;
    debug!(bid = %parsed.bid, "parsed bid response");
    Ok(parsed)
}

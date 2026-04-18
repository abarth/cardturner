use async_trait::async_trait;
use cardturner::ollama::{LlmClient, LlmError};
use cardturner::{bid, Auction, Call, Hand, Seat, Strain, Vulnerability};
use std::sync::Mutex;

struct MockClient {
    response: String,
    captured: Mutex<Option<(String, String)>>,
}

impl MockClient {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            captured: Mutex::new(None),
        }
    }
}

#[async_trait]
impl LlmClient for MockClient {
    async fn chat(&self, system: &str, user: &str) -> Result<String, LlmError> {
        *self.captured.lock().unwrap() = Some((system.to_string(), user.to_string()));
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn bid_parses_structured_response() {
    let hand: Hand = "S:KQ4 H:AJ9 D:KT76 C:KQ3".parse().unwrap();
    let auction = Auction::parse(Seat::N, Vulnerability::None, "").unwrap();
    let client = MockClient::new(r#"{"bid":"1NT","reason":"15 HCP balanced."}"#);

    let resp = bid(&hand, &auction, "system prompt here", &client).await.unwrap();
    assert_eq!(resp.bid, "1NT");
    assert_eq!(resp.reason, "15 HCP balanced.");

    assert_eq!(
        resp.parsed_call().unwrap(),
        Call::Bid {
            level: 1,
            strain: Strain::Notrump
        }
    );

    let captured = client.captured.lock().unwrap();
    let (sys, user) = captured.as_ref().unwrap();
    assert_eq!(sys, "system prompt here");
    assert!(user.contains("Dealer: N"));
    assert!(user.contains("HCP: 18"));
}

#[tokio::test]
async fn bid_surfaces_bad_json() {
    let hand: Hand = "S:KQ4 H:AJ9 D:KT76 C:KQ3".parse().unwrap();
    let auction = Auction::parse(Seat::N, Vulnerability::None, "").unwrap();
    let client = MockClient::new("not json at all");

    let err = bid(&hand, &auction, "sys", &client).await.unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("not json at all"), "raw text should be in error: {msg}");
}

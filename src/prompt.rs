use crate::auction::Auction;
use crate::hand::Hand;

/// Builds the user message sent to the LLM. The system message comes from
/// `prompts/sayc.md`. Keep this output stable — it's snapshot-tested.
pub fn build_user_message(hand: &Hand, auction: &Auction) -> String {
    let next = auction.next_to_bid();
    let dist = hand.distribution();
    format!(
        "Bidding situation:\n\
         - Dealer: {dealer}\n\
         - Vulnerability: {vul}\n\
         - You are sitting: {seat}\n\
         \n\
         Auction so far:\n\
         {table}\n\
         Your hand:\n\
         {pretty}\n\
         - HCP: {hcp}\n\
         - Distribution (S-H-D-C): {s}-{h}-{d}-{c}\n\
         - Compact: {compact}\n\
         \n\
         What is your call? Respond with a single JSON object of the form\n\
         {{\"bid\": \"<call>\", \"reason\": \"<one or two sentences>\"}}\n\
         where <call> is one of: \"Pass\", \"X\" (double), \"XX\" (redouble),\n\
         or a contract like \"1NT\", \"4S\", \"3D\". No prose outside the JSON.",
        dealer = auction.dealer.label(),
        vul = auction.vul.label(),
        seat = next.label(),
        table = auction.pretty_table(),
        pretty = hand.pretty(),
        hcp = hand.hcp(),
        s = dist[0],
        h = dist[1],
        d = dist[2],
        c = dist[3],
        compact = hand,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auction::{Seat, Vulnerability};

    #[test]
    fn message_mentions_required_fields() {
        let hand: Hand = "S:AKQ4 H:KQJ D:KT9 C:K32".parse().unwrap();
        let auction = Auction::parse(Seat::N, Vulnerability::None, "").unwrap();
        let msg = build_user_message(&hand, &auction);
        assert!(msg.contains("Dealer: N"));
        assert!(msg.contains("Vulnerability: None"));
        // S:AKQ4 H:KQJ D:KT9 C:K32 → 9 + 6 + 3 + 3 = 21 HCP
        assert!(msg.contains("HCP: 21"));
        assert!(msg.contains("\"bid\""));
    }
}

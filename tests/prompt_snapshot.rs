use cardturner::prompt::build_user_message;
use cardturner::{Auction, Hand, Seat, Vulnerability};

#[test]
fn snapshot_user_message_for_opening_seat() {
    let hand: Hand = "S:KQ4 H:AJ9 D:KT76 C:KQ3".parse().unwrap();
    let auction = Auction::parse(Seat::N, Vulnerability::None, "").unwrap();
    let msg = build_user_message(&hand, &auction);
    insta::assert_snapshot!("opening_seat_north", msg);
}

#[test]
fn snapshot_user_message_after_partner_opens() {
    let hand: Hand = "S:Q765 H:KJ32 D:T9 C:A54".parse().unwrap();
    let auction = Auction::parse(Seat::N, Vulnerability::NS, "1NT P").unwrap();
    let msg = build_user_message(&hand, &auction);
    insta::assert_snapshot!("response_to_1nt_south", msg);
}

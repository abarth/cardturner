use cardturner::{Auction, Call, Seat, Strain, Vulnerability};

#[test]
fn parses_simple_auction() {
    let a = Auction::parse(Seat::N, Vulnerability::None, "1S P 2H P").unwrap();
    assert_eq!(
        a.calls,
        vec![
            Call::Bid {
                level: 1,
                strain: Strain::Spades
            },
            Call::Pass,
            Call::Bid {
                level: 2,
                strain: Strain::Hearts
            },
            Call::Pass,
        ]
    );
}

#[test]
fn next_to_bid_after_two_rounds() {
    // dealer N, 4 calls -> next is N again
    let a = Auction::parse(Seat::N, Vulnerability::Both, "P P P P").unwrap();
    assert_eq!(a.next_to_bid(), Seat::N);
}

#[test]
fn empty_auction_returns_dealer() {
    let a = Auction::parse(Seat::W, Vulnerability::EW, "").unwrap();
    assert_eq!(a.next_to_bid(), Seat::W);
}

#[test]
fn invalid_call_is_rejected() {
    let err = Auction::parse(Seat::N, Vulnerability::None, "1S Q").unwrap_err();
    assert!(format!("{err}").contains("Q"));
}

#[test]
fn pretty_table_starts_with_header() {
    let a = Auction::parse(Seat::E, Vulnerability::None, "P").unwrap();
    let t = a.pretty_table();
    assert!(t.starts_with("    N     E     S     W"));
}

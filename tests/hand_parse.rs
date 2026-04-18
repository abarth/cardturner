use cardturner::{Hand, HandParseError};

#[test]
fn hcp_and_distribution_for_balanced_hand() {
    let h: Hand = "S:KQ4 H:AJ9 D:KT76 C:KQ3".parse().unwrap();
    assert_eq!(h.hcp(), 18);
    assert_eq!(h.distribution(), [3, 3, 4, 3]);
}

#[test]
fn round_trips_through_display() {
    // 4 + 3 + 3 + 3 = 13
    let original = "S:AKQ4 H:JT9 D:876 C:K32";
    let parsed: Hand = original.parse().unwrap();
    assert_eq!(parsed.to_string(), original);
}

#[test]
fn rejects_too_many_cards() {
    // 5 + 4 + 4 + 3 = 16
    let err: HandParseError = "S:AKQ43 H:JT98 D:8765 C:K32".parse::<Hand>().unwrap_err();
    assert!(matches!(err, HandParseError::WrongCardCount(16)));
}

#[test]
fn rejects_duplicate_suit_token() {
    let err: HandParseError = "S:AKQ4 S:JT9 D:876 C:K32".parse::<Hand>().unwrap_err();
    assert!(matches!(err, HandParseError::DuplicateSuit('S')));
}

use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub fn symbol(self) -> char {
        match self {
            Suit::Spades => '\u{2660}',
            Suit::Hearts => '\u{2665}',
            Suit::Diamonds => '\u{2666}',
            Suit::Clubs => '\u{2663}',
        }
    }

    pub fn letter(self) -> char {
        match self {
            Suit::Spades => 'S',
            Suit::Hearts => 'H',
            Suit::Diamonds => 'D',
            Suit::Clubs => 'C',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn from_char(c: char) -> Option<Rank> {
        Some(match c.to_ascii_uppercase() {
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            '9' => Rank::Nine,
            'T' | '1' => Rank::Ten,
            'J' => Rank::Jack,
            'Q' => Rank::Queen,
            'K' => Rank::King,
            'A' => Rank::Ace,
            _ => return None,
        })
    }

    pub fn letter(self) -> char {
        match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }

    pub fn hcp(self) -> u8 {
        match self {
            Rank::Ace => 4,
            Rank::King => 3,
            Rank::Queen => 2,
            Rank::Jack => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    pub spades: Vec<Rank>,
    pub hearts: Vec<Rank>,
    pub diamonds: Vec<Rank>,
    pub clubs: Vec<Rank>,
}

impl Hand {
    pub fn suit(&self, s: Suit) -> &[Rank] {
        match s {
            Suit::Spades => &self.spades,
            Suit::Hearts => &self.hearts,
            Suit::Diamonds => &self.diamonds,
            Suit::Clubs => &self.clubs,
        }
    }

    pub fn hcp(&self) -> u8 {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .flat_map(|&s| self.suit(s).iter().copied())
            .map(|r| r.hcp())
            .sum()
    }

    pub fn distribution(&self) -> [usize; 4] {
        [
            self.spades.len(),
            self.hearts.len(),
            self.diamonds.len(),
            self.clubs.len(),
        ]
    }

    /// Compact one-line rendering with suit symbols, suitable for the prompt.
    pub fn pretty(&self) -> String {
        let render = |suit: Suit, ranks: &[Rank]| {
            let body: String = if ranks.is_empty() {
                "—".to_string()
            } else {
                ranks.iter().map(|r| r.letter()).collect()
            };
            format!("{} {}", suit.symbol(), body)
        };
        format!(
            "{}  {}  {}  {}",
            render(Suit::Spades, &self.spades),
            render(Suit::Hearts, &self.hearts),
            render(Suit::Diamonds, &self.diamonds),
            render(Suit::Clubs, &self.clubs),
        )
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let render = |ranks: &[Rank]| -> String {
            if ranks.is_empty() {
                "-".into()
            } else {
                ranks.iter().map(|r| r.letter()).collect()
            }
        };
        write!(
            f,
            "S:{} H:{} D:{} C:{}",
            render(&self.spades),
            render(&self.hearts),
            render(&self.diamonds),
            render(&self.clubs),
        )
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HandParseError {
    #[error("missing suit '{0}' (expected tokens like S:..., H:..., D:..., C:...)")]
    MissingSuit(char),
    #[error("duplicate suit '{0}'")]
    DuplicateSuit(char),
    #[error("unknown suit prefix '{0}'")]
    UnknownSuit(String),
    #[error("malformed token '{0}': expected '<S|H|D|C>:<ranks>'")]
    MalformedToken(String),
    #[error("invalid rank character '{0}' in suit '{1}'")]
    InvalidRank(char, char),
    #[error("duplicate card {1}{0} in hand")]
    DuplicateCard(char, char),
    #[error("hand has {0} cards, expected 13")]
    WrongCardCount(usize),
}

impl FromStr for Hand {
    type Err = HandParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut spades: Option<Vec<Rank>> = None;
        let mut hearts: Option<Vec<Rank>> = None;
        let mut diamonds: Option<Vec<Rank>> = None;
        let mut clubs: Option<Vec<Rank>> = None;

        for token in input.split_whitespace() {
            let (prefix, rest) = token
                .split_once(':')
                .ok_or_else(|| HandParseError::MalformedToken(token.to_string()))?;
            if prefix.len() != 1 {
                return Err(HandParseError::UnknownSuit(prefix.to_string()));
            }
            let suit_letter = prefix.chars().next().unwrap().to_ascii_uppercase();
            let slot: &mut Option<Vec<Rank>> = match suit_letter {
                'S' => &mut spades,
                'H' => &mut hearts,
                'D' => &mut diamonds,
                'C' => &mut clubs,
                other => return Err(HandParseError::UnknownSuit(other.to_string())),
            };
            if slot.is_some() {
                return Err(HandParseError::DuplicateSuit(suit_letter));
            }
            let mut seen = HashSet::new();
            let mut ranks = Vec::with_capacity(rest.len());
            for ch in rest.chars() {
                if ch == '-' && rest.len() == 1 {
                    break; // explicit void
                }
                let rank = Rank::from_char(ch)
                    .ok_or(HandParseError::InvalidRank(ch, suit_letter))?;
                if !seen.insert(rank) {
                    return Err(HandParseError::DuplicateCard(rank.letter(), suit_letter));
                }
                ranks.push(rank);
            }
            ranks.sort_by(|a, b| b.cmp(a));
            *slot = Some(ranks);
        }

        let spades = spades.ok_or(HandParseError::MissingSuit('S'))?;
        let hearts = hearts.ok_or(HandParseError::MissingSuit('H'))?;
        let diamonds = diamonds.ok_or(HandParseError::MissingSuit('D'))?;
        let clubs = clubs.ok_or(HandParseError::MissingSuit('C'))?;

        let total = spades.len() + hearts.len() + diamonds.len() + clubs.len();
        if total != 13 {
            return Err(HandParseError::WrongCardCount(total));
        }

        Ok(Hand {
            spades,
            hearts,
            diamonds,
            clubs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_hand() {
        let hand: Hand = "S:AKQ4 H:JT9 D:876 C:K32".parse().unwrap();
        assert_eq!(hand.distribution(), [4, 3, 3, 3]);
        // A+K+Q + J + K = 4+3+2+1+3 = 13
        assert_eq!(hand.hcp(), 13);
    }

    #[test]
    fn accepts_lowercase_input() {
        let hand: Hand = "s:akq4 h:jt9 d:876 c:k32".parse().unwrap();
        assert_eq!(hand.spades, vec![Rank::Ace, Rank::King, Rank::Queen, Rank::Four]);
        assert_eq!(hand.hearts, vec![Rank::Jack, Rank::Ten, Rank::Nine]);
    }

    #[test]
    fn ten_can_be_written_as_1() {
        let hand: Hand = "S:AKQ4 H:J19 D:876 C:K32".parse().unwrap();
        assert_eq!(hand.hearts, vec![Rank::Jack, Rank::Ten, Rank::Nine]);
    }

    #[test]
    fn rejects_wrong_count() {
        // 3 + 3 + 3 + 3 = 12
        let err = "S:AKQ H:JT9 D:876 C:K32".parse::<Hand>().unwrap_err();
        assert!(matches!(err, HandParseError::WrongCardCount(12)));
    }

    #[test]
    fn empty_suit_token_is_a_void() {
        // 5 + 3 + 5 + 0 = 13
        let hand: Hand = "S:AKQ43 H:JT9 D:87654 C:".parse().unwrap();
        assert_eq!(hand.distribution(), [5, 3, 5, 0]);
    }

    #[test]
    fn rejects_duplicate_card_in_suit() {
        let err = "S:AAKQ4 H:JT9 D:8765 C:K3".parse::<Hand>().unwrap_err();
        assert!(matches!(err, HandParseError::DuplicateCard(_, _)));
    }

    #[test]
    fn explicit_void_dash() {
        let hand: Hand = "S:- H:AKQJT98765432 D:- C:-".parse().unwrap();
        assert_eq!(hand.distribution(), [0, 13, 0, 0]);
        assert_eq!(hand.hcp(), 10);
    }

    #[test]
    fn display_round_trip() {
        let hand: Hand = "S:AKQ4 H:JT9 D:876 C:K32".parse().unwrap();
        let s = hand.to_string();
        let again: Hand = s.parse().unwrap();
        assert_eq!(hand, again);
    }
}

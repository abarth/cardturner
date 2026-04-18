use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    Notrump,
}

impl Strain {
    pub fn label(self) -> &'static str {
        match self {
            Strain::Clubs => "C",
            Strain::Diamonds => "D",
            Strain::Hearts => "H",
            Strain::Spades => "S",
            Strain::Notrump => "NT",
        }
    }

    fn from_str(s: &str) -> Option<Strain> {
        Some(match s.to_ascii_uppercase().as_str() {
            "C" => Strain::Clubs,
            "D" => Strain::Diamonds,
            "H" => Strain::Hearts,
            "S" => Strain::Spades,
            "N" | "NT" => Strain::Notrump,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Call {
    Pass,
    Double,
    Redouble,
    Bid { level: u8, strain: Strain },
}

impl Call {
    pub fn parse(token: &str) -> Result<Call, AuctionError> {
        let upper = token.to_ascii_uppercase();
        match upper.as_str() {
            "P" | "PASS" => return Ok(Call::Pass),
            "X" | "DBL" | "DOUBLE" => return Ok(Call::Double),
            "XX" | "RDBL" | "REDOUBLE" => return Ok(Call::Redouble),
            _ => {}
        }
        let mut chars = upper.chars();
        let level_char = chars
            .next()
            .ok_or_else(|| AuctionError::InvalidCall(token.to_string()))?;
        let level = level_char
            .to_digit(10)
            .filter(|d| (1..=7).contains(d))
            .ok_or_else(|| AuctionError::InvalidCall(token.to_string()))?
            as u8;
        let strain_str: String = chars.collect();
        let strain = Strain::from_str(&strain_str)
            .ok_or_else(|| AuctionError::InvalidCall(token.to_string()))?;
        Ok(Call::Bid { level, strain })
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Call::Pass => write!(f, "Pass"),
            Call::Double => write!(f, "X"),
            Call::Redouble => write!(f, "XX"),
            Call::Bid { level, strain } => write!(f, "{}{}", level, strain.label()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    N,
    E,
    S,
    W,
}

impl Seat {
    pub fn label(self) -> &'static str {
        match self {
            Seat::N => "N",
            Seat::E => "E",
            Seat::S => "S",
            Seat::W => "W",
        }
    }

    pub fn next(self) -> Seat {
        match self {
            Seat::N => Seat::E,
            Seat::E => Seat::S,
            Seat::S => Seat::W,
            Seat::W => Seat::N,
        }
    }
}

impl FromStr for Seat {
    type Err = AuctionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_uppercase().as_str() {
            "N" | "NORTH" => Seat::N,
            "E" | "EAST" => Seat::E,
            "S" | "SOUTH" => Seat::S,
            "W" | "WEST" => Seat::W,
            other => return Err(AuctionError::InvalidSeat(other.to_string())),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vulnerability {
    None,
    NS,
    EW,
    Both,
}

impl Vulnerability {
    pub fn label(self) -> &'static str {
        match self {
            Vulnerability::None => "None",
            Vulnerability::NS => "N/S",
            Vulnerability::EW => "E/W",
            Vulnerability::Both => "Both",
        }
    }
}

impl FromStr for Vulnerability {
    type Err = AuctionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_uppercase().replace('/', "").as_str() {
            "NONE" | "NIL" | "-" | "" => Vulnerability::None,
            "NS" => Vulnerability::NS,
            "EW" => Vulnerability::EW,
            "BOTH" | "ALL" => Vulnerability::Both,
            other => return Err(AuctionError::InvalidVulnerability(other.to_string())),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auction {
    pub dealer: Seat,
    pub vul: Vulnerability,
    pub calls: Vec<Call>,
}

impl Auction {
    pub fn parse(dealer: Seat, vul: Vulnerability, s: &str) -> Result<Self, AuctionError> {
        let calls: Result<Vec<Call>, _> = s.split_whitespace().map(Call::parse).collect();
        Ok(Auction {
            dealer,
            vul,
            calls: calls?,
        })
    }

    pub fn next_to_bid(&self) -> Seat {
        let mut seat = self.dealer;
        for _ in 0..self.calls.len() {
            seat = seat.next();
        }
        seat
    }

    /// Render the auction as a four-column N/E/S/W table.
    /// Cells before the dealer are blank. The next-to-bid cell is "?".
    pub fn pretty_table(&self) -> String {
        let leading = match self.dealer {
            Seat::N => 0,
            Seat::E => 1,
            Seat::S => 2,
            Seat::W => 3,
        };

        let mut cells: Vec<String> = Vec::new();
        for _ in 0..leading {
            cells.push(".".to_string());
        }
        for c in &self.calls {
            cells.push(c.to_string());
        }
        cells.push("?".to_string());

        let mut out = String::from("    N     E     S     W\n");
        for (i, cell) in cells.iter().enumerate() {
            if i % 4 == 0 {
                out.push_str("  ");
            }
            out.push_str(&format!("{:<6}", cell));
            if i % 4 == 3 {
                out.push('\n');
            }
        }
        if cells.len() % 4 != 0 {
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuctionError {
    #[error("invalid call '{0}' (expected Pass/X/XX or like 1NT, 4S)")]
    InvalidCall(String),
    #[error("invalid seat '{0}' (expected N/E/S/W)")]
    InvalidSeat(String),
    #[error("invalid vulnerability '{0}' (expected None, NS, EW, Both)")]
    InvalidVulnerability(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_calls() {
        assert_eq!(Call::parse("P").unwrap(), Call::Pass);
        assert_eq!(Call::parse("Pass").unwrap(), Call::Pass);
        assert_eq!(Call::parse("X").unwrap(), Call::Double);
        assert_eq!(Call::parse("XX").unwrap(), Call::Redouble);
        assert_eq!(
            Call::parse("1NT").unwrap(),
            Call::Bid {
                level: 1,
                strain: Strain::Notrump
            }
        );
        assert_eq!(
            Call::parse("4s").unwrap(),
            Call::Bid {
                level: 4,
                strain: Strain::Spades
            }
        );
        assert!(Call::parse("8H").is_err());
        assert!(Call::parse("1Z").is_err());
    }

    #[test]
    fn auction_next_to_bid() {
        let a = Auction::parse(Seat::N, Vulnerability::None, "1S P 2H P").unwrap();
        // N E S W → after 4 calls back to N
        assert_eq!(a.next_to_bid(), Seat::N);
        let b = Auction::parse(Seat::E, Vulnerability::Both, "").unwrap();
        assert_eq!(b.next_to_bid(), Seat::E);
        let c = Auction::parse(Seat::S, Vulnerability::EW, "1H").unwrap();
        assert_eq!(c.next_to_bid(), Seat::W);
    }

    #[test]
    fn vulnerability_parsing() {
        assert_eq!("None".parse::<Vulnerability>().unwrap(), Vulnerability::None);
        assert_eq!("ns".parse::<Vulnerability>().unwrap(), Vulnerability::NS);
        assert_eq!("E/W".parse::<Vulnerability>().unwrap(), Vulnerability::EW);
        assert_eq!("Both".parse::<Vulnerability>().unwrap(), Vulnerability::Both);
        assert!("nope".parse::<Vulnerability>().is_err());
    }
}

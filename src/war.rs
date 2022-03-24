//! Defines the rules of war. Exactly, how a turn resolves (rather than how it
//! flows, see [`crate::game_flow`] for that).
//!
//! The module defines how to parse a [`Card`] definition from text, this is to
//! make it possible to load decks from file.
//!
//! # Rules
//!
//! * The card with the highest score wins except:
//!   * Zero beats Nine
//!   * the `Zihbm` reverses the outcome of the turn (including Zero/Nine
//!     interaction)
//! * The winner gets the two card, and their face value count toward their
//!   score.
//! * If the two cards have the same value, each player get back their card,
//!   gaining the same score.
//!
//! ## Effects
//!
//! Cards have optional effects, called [`WordOfPower`]s. The effects are
//! listed in the enum definition.
use std::str::FromStr;

use bevy::prelude::{Color, Component};
#[cfg(feature = "debug")]
use bevy_inspector_egui::Inspectable;
use enum_map::Enum;

#[derive(Clone, PartialEq, Debug)]
pub enum ParseError {
    BadValue(String),
    BadWord(String),
    EmptyWord,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyWord => write!(f, "The word is specified as non-existing"),
            ParseError::BadWord(word) => write!(f, "The word {word} is invalid"),
            ParseError::BadValue(value) => write!(f, "The value {value} is invalid"),
        }
    }
}
impl std::error::Error for ParseError {}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum BattleOutcome {
    Loss,
    Tie,
    Win,
}

/// Card point value.
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
}
impl Value {
    #[rustfmt::skip]
    pub fn beats(&self, other: &Self) -> BattleOutcome {
        use BattleOutcome::*;
        use Value::*;
        match (self, other) {
            (a, b) if a == b => Tie,
            (Zero,  Nine) => Win,
            (One,   Zero) => Win,
            (Two,   Zero | One) => Win,
            (Three, Zero | One | Two) => Win,
            (Four,  Zero | One | Two | Three) => Win,

            (Zero | One | Two | Three | Four, _) => Loss,

            (Five,  Nine | Eight | Seven | Six) => Loss,
            (Six,   Nine | Eight | Seven) => Loss,
            (Seven, Nine | Eight) => Loss,
            (Eight, Nine) => Loss,

            (Nine,  Zero) => Loss,
            (Five | Six | Seven | Eight | Nine, _) => Win,
        }
    }
}
impl FromStr for Value {
    type Err = ParseError;
    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Value::*;
        match s {
            "0" => Ok(Zero),  "1" => Ok(One),   "2" => Ok(Two),
            "3" => Ok(Three), "4" => Ok(Four),  "5" => Ok(Five),
            "6" => Ok(Six),   "7" => Ok(Seven), "8" => Ok(Eight),
            "9" => Ok(Nine),  _ => Err(ParseError::BadValue(s.to_owned())),
        }
    }
}

/// Additional effects of cards.
///
/// ## Card effects
///
/// * `Egeq`: Give an extra seed to the player.
/// * `Qube`: Double points.
/// * `Geh`: Card of [`Value::Zero`] earns 12 points.
/// * `Zihbm`: The winner is swapped.
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Enum, Clone, Copy, Debug, PartialEq)]
pub enum WordOfPower {
    Egeq,
    Qube,
    Zihbm,
    Geh,
    Het,
    Meb,
}
impl WordOfPower {
    pub fn color(self) -> Color {
        use WordOfPower::*;
        match self {
            Egeq => Color::LIME_GREEN,
            Geh => Color::CYAN,
            Het => Color::PURPLE,
            Meb => Color::GRAY,
            Qube => Color::GOLD,
            Zihbm => Color::PINK,
        }
    }
    pub fn flavor_text(&self) -> &'static str {
        use WordOfPower::*;
        match self {
            Egeq => "Gain a seed",
            Qube => "Double points",
            Zihbm => "Swap winners",
            Geh => "Zero earns 12",
            _ => "Unimplemented",
        }
    }
}
impl FromStr for WordOfPower {
    type Err = ParseError;
    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WordOfPower::*;
        match s {
            "seed" | "s" => Ok(Egeq),  "doub" | "d" => Ok(Qube),
            "swap" | "w" => Ok(Zihbm), "zero" | "z" => Ok(Geh),
            "____" | "_" => Err(ParseError::EmptyWord),
            "het" => Ok(Het), "meb" => Ok(Meb),
            _ => Err(ParseError::BadWord(s.to_owned())),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component, Clone, Debug)]
#[non_exhaustive]
pub struct Card {
    pub word: Option<WordOfPower>,
    pub value: Value,
}
impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word && self.value == other.value
    }
}
impl Eq for Card {}
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
#[cfg(feature = "debug")]
impl Default for Card {
    fn default() -> Self {
        Self { word: None, value: Value::Zero }
    }
}
impl FromStr for Card {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseError::EmptyWord;
        let (value, word) = s.split_at(1);
        let word = word.parse().map_or_else(
            |err| if matches!(err, EmptyWord) { Ok(None) } else { Err(err) },
            |word| Ok(Some(word)),
        );
        Ok(Card { value: value.parse()?, word: word? })
    }
}
impl Card {
    pub fn beats(&self, other: &Self) -> BattleOutcome {
        use BattleOutcome::{Loss, Tie, Win};
        use WordOfPower::Zihbm;
        let swaps = |card: &Card| card.word == Some(Zihbm);
        match (self.value.beats(&other.value), swaps(self) ^ swaps(other)) {
            (Loss, false) | (Win, true) => Loss,
            (Loss, true) | (Win, false) => Win,
            (Tie, _) => Tie,
        }
    }
    pub fn max_value(&self) -> i32 {
        let value = self.value as i32;
        let word_max_bonus = match self.word {
            // Zero = 12
            Some(WordOfPower::Geh) => 12,
            // Double card value (including opponent's)
            Some(WordOfPower::Qube) => value + 9,
            _ => 0,
        };
        word_max_bonus + value
    }
    pub fn bonus_points(&self, other: &Self) -> (i32, i32) {
        use Value::Zero;
        use WordOfPower::{Geh, Qube};
        let is_word = |c: &Self, word| (c.word == Some(word)) as i32;
        let is_zero = |c: &Self| if c.value == Zero { 1 } else { 0 };
        let zero_bonus = 12 * (is_word(self, Geh) + is_word(other, Geh));
        let zero_bonus = |c| is_zero(c) * zero_bonus;
        let mul_bonus = is_word(self, Qube) + is_word(other, Qube);
        (
            zero_bonus(self) * (mul_bonus + 1) + self.value as i32 * mul_bonus,
            zero_bonus(other) * (mul_bonus + 1) + other.value as i32 * mul_bonus,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! bonus_for {
        ($lcard:tt, $rcard:tt) => {{
            let lcard: Card = stringify!($lcard).parse().unwrap();
            let rcard: Card = stringify!($rcard).parse().unwrap();
            lcard.bonus_points(&rcard)
        }};
    }
    #[test]
    fn bonus_point_test() {
        assert_eq!((0, 0), bonus_for!(9_, 9_));
        assert_eq!((12, 0), bonus_for!(0z, 9_));
        assert_eq!((12, 0), bonus_for!(0_, 9z));
        assert_eq!((24, 0), bonus_for!(0z, 9z));
        assert_eq!((24, 9), bonus_for!(0z, 9d));
        assert_eq!((24, 9), bonus_for!(0d, 9z));
        assert_eq!((0, 2), bonus_for!(0d, 1d));
        assert_eq!((1, 1), bonus_for!(1d, 1_));
        assert_eq!((2, 2), bonus_for!(1d, 1d));
    }
}

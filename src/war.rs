//! Defines the rules of war

use bevy::prelude::{Color, Component};
#[cfg(feature = "debug")]
use bevy_inspector_egui::Inspectable;
use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum BattleOutcome {
    Loss,
    Tie,
    Win,
}

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
    pub fn new(word: Option<WordOfPower>, value: Value) -> Self {
        Self { word, value }
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
}

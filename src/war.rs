//! Defines the rules of war
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
#[derive(Enum, Debug, Clone, Copy, PartialEq)]
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

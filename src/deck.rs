use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{
    card::{Card, WordOfPower},
    war::Value,
};

struct Deck {
    cards: Vec<Card>,
}
impl Deck {
    fn new(mut cards: Vec<Card>) -> Self {
        cards.reverse();
        Self { cards }
    }
    #[allow(unused)]
    fn peek(&self, count: usize) -> &[Card] {
        let len = self.remaining();
        let index = len - count.min(len);
        &self.cards[..index]
    }
    fn draw(&mut self, count: usize) -> Vec<Card> {
        let len = self.remaining();
        let index = len - count.min(len);
        self.cards.split_off(index)
    }
    fn remaining(&self) -> usize {
        self.cards.len()
    }
}

macro_rules! impl_deck_methods {
    ($what:ident) => (
        impl $what {
            // impl_deck_methods!(@method peek((&), count: usize) -> &[Card]);
            impl_deck_methods!(@method draw((&mut), count: usize) -> Vec<Card>);
            impl_deck_methods!(@method remaining((&)) -> usize);
        }
    );
    (@method $name:ident (
        ($($self_param:tt)*)
        $(, $param_name:ident : $param_type:ty)*)
        $(-> $ret:ty)?
    ) => (
        pub fn $name($($self_param)* self $(, $param_name : $param_type)*) $(-> $ret)? {
            self.0.$name($($param_name ,)*)
        }
    )
}

macro_rules! cards {
    ($($value:ident $word:ident |)*) => (
        Deck::new(vec![ $( Card::new(WordOfPower::$word, Value::$value) ,)* ])
    )
}

pub struct PlayerDeck(Deck);
impl_deck_methods!(PlayerDeck);
impl PlayerDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            Zero Egeq  | One Qube | Two Qube   |
            Three Egeq | Four Egeq | Five Qube |
            Zero Qube  | Three Meb | Two Geh   |
        ])
    }
}

pub struct OppoDeck(Deck);
impl_deck_methods!(OppoDeck);
impl OppoDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            Nine Zihbm | Eight Qube | Seven Geh |
            Six Egeq   | Five Qube  | Nine Egeq |
            Five Zihbm | Eight Geh  | Four Meb  |
        ])
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OppoDeck::new())
            .insert_resource(PlayerDeck::new());
    }
}

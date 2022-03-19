#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GameState {
    MainMenu,
    /// Wait until the game scene is fully loaded if not already
    WaitLoaded,
    /// The game is running
    Playing,
    /// Restart menu after gameover
    RestartMenu,
}

// LEAD: potential improvement: logic in game_flow really does not care for the
// distinction between player turn and oppo turn, we might be able to merge
// Oppo and Player
/// In-game game state, see the diagram in [`crate::game_flow`] for flow chart.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TurnState {
    /// Game just started
    Starting,
    /// Participants must draw 3 cards
    Draw,
    /// A new turn has begun
    New,
    /// Player's turn to play a card
    Player,
    /// Oppo's turn to select a card
    Oppo,
    /// A participants has played a card
    CardPlayed,
}

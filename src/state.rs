#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GameState {
    MainMenu,
    /// Wait until the game scene is fully loaded if not already
    WaitSceneLoaded,
    /// The game is running
    Playing,
    /// Restart menu after gameover
    RestartMenu,
}

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
    /// Player has played a card
    PlayerActivated,
    /// Oppo's turn to select a card
    Oppo,
    /// Oppo has played his card
    OppoActivated,
}

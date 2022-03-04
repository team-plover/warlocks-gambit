#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GameState {
    ScenePreload,
    MainMenu,
    LoadScene,
    Playing,
    /// Gameover animation
    GameOver,
    /// Restart menu after gameover
    RestartMenu,
    /// In-game settings
    PauseMenu,
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

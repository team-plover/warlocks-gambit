#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GameState {
    MainMenu,
    LoadScene,
    Playing,
    /// Gameover animation
    GameOver,
    /// Restart menu after gameover
    RestartMenu,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TurnState {
    Player,
    PlayerActivated,
    Oppo,
    OppoActivated,
    New,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GameState {
    MainMenu,
    Playing,
    /// Gameover animation
    GameOver,
    /// Restart menu after gameover
    RestartMenu,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Winner {
    None,
    Me,
    Other,
    Draw,
}

impl Default for Winner {
    fn default() -> Self {
        Winner::None
    }
}

#[derive(PartialEq, Eq, Clone, Default)]
pub struct QuitGame(pub bool);

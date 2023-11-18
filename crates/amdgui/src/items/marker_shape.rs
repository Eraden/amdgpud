#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkerShape {
    Circle,
    Diamond,
    Square,
    Cross,
    Plus,
    Up,
    Down,
    Left,
    Right,
    Asterisk,
}

impl MarkerShape {
    /// Get a vector containing all marker shapes.
    pub fn all() -> impl Iterator<Item = MarkerShape> {
        [
            Self::Circle,
            Self::Diamond,
            Self::Square,
            Self::Cross,
            Self::Plus,
            Self::Up,
            Self::Down,
            Self::Left,
            Self::Right,
            Self::Asterisk,
        ]
        .iter()
        .copied()
    }
}

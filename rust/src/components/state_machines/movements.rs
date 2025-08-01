use godot::builtin::Vector2;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Direction {
    #[default]
    East,
    West,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::East => write!(f, "east"),
            Direction::West => write!(f, "west"),
        }
    }
}
impl Direction {
    pub fn from_vel(velocity: &Vector2) -> Direction {
        if velocity.x < 0.0 {
            Direction::West
        } else {
            Direction::East
        }
    }
}

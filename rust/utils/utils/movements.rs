use godot::builtin::Vector2;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Directions {
    North,
    NorthEast,
    NorthWest,
    #[default]
    East,
    South,
    SouthEast,
    SouthWest,
    West,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum PlatformerDirection {
    #[default]
    East,
    West,
}

impl std::fmt::Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Directions::North => write!(f, "north"),
            Directions::NorthEast => write!(f, "north_east"),
            Directions::NorthWest => write!(f, "north_west"),
            Directions::East => write!(f, "east"),
            Directions::South => write!(f, "south"),
            Directions::SouthEast => write!(f, "south_east"),
            Directions::SouthWest => write!(f, "south_west"),
            Directions::West => write!(f, "west"),
        }
    }
}
impl Directions {
    pub fn to_velocity(&self) -> Vector2 {
        match self {
            Directions::North => Vector2::UP,
            Directions::NorthEast => Vector2::new(1.0, -1.0).normalized(),
            Directions::NorthWest => Vector2::new(-1.0, -1.0).normalized(),
            Directions::East => Vector2::RIGHT,
            Directions::South => Vector2::DOWN,
            Directions::SouthEast => Vector2::new(1.0, 1.0).normalized(),
            Directions::SouthWest => Vector2::new(-1.0, 1.0).normalized(),
            Directions::West => Vector2::LEFT,
        }
    }

    pub fn from_velocity(vel: &Vector2) -> Directions {
        if vel.x > 0.0 && vel.y < 0.0 {
            return Directions::NorthEast;
        }
        if vel.x < 0.0 && vel.y < 0.0 {
            return Directions::NorthWest;
        }
        if vel.x > 0.0 && vel.y > 0.0 {
            return Directions::SouthEast;
        }
        if vel.x < 0.0 && vel.y > 0.0 {
            return Directions::SouthWest;
        }
        if vel.x > 0.0 {
            return Directions::East;
        }
        if vel.x < 0.0 {
            return Directions::West;
        }
        if vel.y < 0.0 {
            Directions::North
        } else {
            Directions::South
        }
    }
}

impl std::fmt::Display for PlatformerDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformerDirection::East => write!(f, "east"),
            PlatformerDirection::West => write!(f, "west"),
        }
    }
}
impl PlatformerDirection {
    pub fn from_platformer_velocity(velocity: &Vector2) -> PlatformerDirection {
        if velocity.x < 0.0 {
            PlatformerDirection::West
        } else {
            PlatformerDirection::East
        }
    }
}

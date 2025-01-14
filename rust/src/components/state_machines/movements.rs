use godot::builtin::Vector2;

#[derive(Default)]
pub enum Directions {
    #[default]
    North,
    NorthEast,
    NorthWest,
    East,
    South,
    SouthEast,
    SouthWest,
    West,
}

impl ToString for Directions {
    fn to_string(&self) -> String {
        match self {
            Directions::North => "north".to_string(),
            Directions::NorthEast => "north_east".to_string(),
            Directions::NorthWest => "north_west".to_string(),
            Directions::East => "east".to_string(),
            Directions::South => "south".to_string(),
            Directions::SouthEast => "south_east".to_string(),
            Directions::SouthWest => "south_west".to_string(),
            Directions::West => "west".to_string(),
        }
    }
}
impl Directions {
    pub fn get_direction_from_velocity(vel: Vector2) -> Directions {
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

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

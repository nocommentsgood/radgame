use godot::builtin::real;

#[derive(Default, Debug, Clone)]
pub struct SpeedComponent {
    pub attack: real,
    pub patrol: real,
    pub aggro: real,
}

impl SpeedComponent {
    pub fn new(attack: real, patrol: real, aggro: real) -> Self {
        Self {
            attack,
            patrol,
            aggro,
        }
    }
}

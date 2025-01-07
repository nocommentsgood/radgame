use std::fmt::Display;

#[derive(Default, PartialEq, Clone, Debug)]
pub enum EnemyState {
    Attacking,
    Moving,
    Dodging,
    #[default]
    Idle,
}

impl Display for EnemyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnemyState::Attacking => write!(f, "attacking"),
            EnemyState::Moving => write!(f, "moving"),
            EnemyState::Dodging => write!(f, "dodge"),
            EnemyState::Idle => write!(f, "idle"),
        }
    }
}

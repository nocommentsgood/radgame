#[derive(Default, PartialEq, Clone, Debug)]
pub enum CharacterState {
    Attacking,
    Running,
    Walking,
    Dodging,
    #[default]
    Idle,
}

impl ToString for CharacterState {
    fn to_string(&self) -> String {
        match self {
            CharacterState::Attacking => "attack".to_string(),
            CharacterState::Running => "run".to_string(),
            CharacterState::Walking => "walk".to_string(),
            CharacterState::Dodging => "dodge".to_string(),
            CharacterState::Idle => "idle".to_string(),
        }
    }
}

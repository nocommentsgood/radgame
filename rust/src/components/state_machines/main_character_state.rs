#[derive(Default, PartialEq, Clone, Debug)]
pub enum CharacterState {
    #[default]
    Default,
    RunningLeft,
    RunningRight,
    RunningUp,
    RunningDown,
    LightAttackLeft,
    LightAttackRight,
    Jumping,
    CastingSpell,
}

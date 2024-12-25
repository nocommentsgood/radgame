#[derive(Default)]
pub enum CharacterState {
    #[default]
    DEFAULT,
    MOVING,
    ATTACKING,
    JUMPING,
    CASTING_SPELL,
}

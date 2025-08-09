#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EnemyTimer {
    AttackAnimation,
    AttackChainCooldown,
    AttackCooldown,
    Idle,
    Patrol,
    #[default]
    None,
}

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

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlayerTimer {
    AttackChain,
    DodgeAnimation,
    JumpingAnimation,
    AttackAnimation,
    AttackAnimation2,
    HealingAnimation,
    HealingCooldown,
    HurtAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
    Coyote,
    DodgeCooldown,
}

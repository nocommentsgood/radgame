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
    WallJumpLimit,
    AttackChain,
    DodgeAnimation,
    JumpingCooldown,
    AttackAnimation,
    Attack2Animation,
    HealingAnimation,
    HealingCooldown,
    HurtAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
    Coyote,
    DodgeCooldown,
    JumpTimeLimit,
}

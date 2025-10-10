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

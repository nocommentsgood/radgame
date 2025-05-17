#[derive(Debug, Default, PartialEq, Clone)]
pub enum Event {
    Wasd,
    WasdJustPressed,
    DodgeButton,
    AttackButton,
    JumpButton,
    ParryButton,
    GrabbedLedge,
    HealingButton,
    FailedFloorCheck,
    ActionReleasedEarly,
    TimerElapsed,
    TimerInProgress,
    OnFloor,
    #[default]
    None,
}
#[derive(Default, Debug)]
pub enum EnemyEvent {
    FoundPlayer,
    FailedFloorCheck,
    OnFloor,
    LostPlayer,
    InAttackRange,
    TimerElapsed,
    #[default]
    None,
}

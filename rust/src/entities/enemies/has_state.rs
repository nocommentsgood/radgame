use super::enemy_state_machine::EnemyStateMachine;
use statig::prelude::StateMachine;

pub trait HasState {
    fn sm_mut(&mut self) -> &mut StateMachine<EnemyStateMachine>;

    fn sm(&self) -> &StateMachine<EnemyStateMachine>;
}

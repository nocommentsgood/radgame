use statig::prelude::StateMachine;

use crate::components::state_machines::enemy_state_machine::EnemyStateMachine;

// I'm not sure what best pratice is, as far as returning mutable references vs. non-mutable
// references, such as this.
pub trait HasState {
    fn sm_mut(&mut self) -> &mut StateMachine<EnemyStateMachine>;

    fn sm(&self) -> &StateMachine<EnemyStateMachine>;
}

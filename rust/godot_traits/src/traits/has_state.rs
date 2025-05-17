use statig::prelude::StateMachine;

// I'm not sure what best pratice is, as far as returning mutable references vs. non-mutable
// references, such as this.
pub trait HasState {
    fn sm_mut(&mut self)
    -> &mut StateMachine<utils::utils::enemy_state_machine::EnemyStateMachine>;

    fn sm(&self) -> &StateMachine<utils::utils::enemy_state_machine::EnemyStateMachine>;
}

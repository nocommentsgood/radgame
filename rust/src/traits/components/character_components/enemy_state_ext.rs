use godot::obj::{Inherits, WithBaseField};

pub trait EnemyStateMachineExt: super::has_state::HasState
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node2D>>,
{
}

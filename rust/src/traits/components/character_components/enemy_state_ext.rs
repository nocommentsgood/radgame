use godot::builtin::Vector2;
use godot::obj::{Gd, Inherits, WithBaseField};

use crate::classes::characters::main_character::MainCharacter;
use crate::classes::components::speed_component::SpeedComponent;
use crate::classes::components::timer_component as timers;
use crate::classes::enemies::patrol_component::PatrolComponent;
use crate::components::state_machines::enemy_state_machine;

use super::moveable::{MoveableCharacter, MoveableEntity};

// TODO: Find a way to conditionally implement traits? Possibly making it generic. Mostly for
// MoveableCharacter versus MoveableEntity

/// Implement on types that use an EnemyStateMachine
/// For types without a base field of CharacterBody2D, see 'EnemyEntityStateMachineExt'.
pub trait EnemyCharacterStateMachineExt: super::has_state::HasState + MoveableCharacter
where
    Self: Inherits<godot::classes::Node2D>
        + WithBaseField<Base: Inherits<godot::classes::CharacterBody2D>>,
{
    fn timers(&mut self) -> &mut timers::EnemyTimers;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> SpeedComponent;
    fn patrol_comp(&self) -> PatrolComponent;

    fn attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers().attack_animation.value;
        let delta = self
            .base()
            .upcast_ref::<godot::classes::Node2D>()
            .get_physics_process_delta_time();
        let speed = self.speeds().attack;
        let velocity = self.get_velocity();
        self.timers().attack_animation.value -= delta;
        self.slide(&velocity, &speed);

        if time <= 0.0 {
            self.timers().attack_animation.reset();
            self.sm_mut().handle(
                &crate::components::state_machines::enemy_state_machine::EnemyEvent::TimerElapsed,
            );
        }
    }

    fn fall(&mut self) {
        let speed = self.speeds().aggro;
        let velocity = Vector2::DOWN;
        self.slide(&velocity, &speed);

        if self.base().is_on_floor() {
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::OnFloor);
        }
    }

    fn chain_attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers().chain_attack.value;
        let delta = self.base().get_physics_process_delta_time();
        let velocity = self.get_velocity();
        let speed = self.speeds().attack;
        self.timers().chain_attack.value -= delta;
        self.slide(&velocity, &speed);

        if time <= 0.0 {
            self.timers().chain_attack.reset();
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let time = self.timers().patrol.value;
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity();
        let delta = self.base().get_physics_process_delta_time();

        self.update_direction();
        self.slide(&velocity, &speed);
        self.timers().patrol.value -= delta;

        if time <= 0.0 {
            self.timers().patrol.reset();
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let time = self.timers().idle.value;
        let delta = self.base().get_physics_process_delta_time();
        let velocity = Vector2::ZERO;
        self.slide(&velocity, &0.0);
        self.timers().idle.value -= delta;

        if time <= 0.0 {
            self.timers().idle.reset();
            self.set_velocity(
                self.patrol_comp()
                    .get_furthest_distance(self.base().get_global_position()),
            );
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self
            .base()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AttackArea");
        let delta = self.base().get_physics_process_delta_time();
        let speed = self.speeds().aggro;
        let player_position = player.get_position();
        let velocity = Vector2::new(
            self.base().get_position().direction_to(player_position).x,
            0.0,
        );
        self.set_velocity(velocity);
        self.update_direction();
        self.slide(&velocity, &speed);

        if attack_range.has_overlapping_bodies()
            && self.timers().attack_cooldown.value == self.timers().attack_cooldown.initial_value()
        {
            self.timers().attack_cooldown.value -= delta;
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }
}

pub trait EnemyEntityStateMachineExt: super::has_state::HasState + MoveableEntity
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node2D>>,
{
    fn timers(&mut self) -> &mut timers::EnemyTimers;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> SpeedComponent;
    fn patrol_comp(&self) -> PatrolComponent;

    fn attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers().attack_animation.value;
        let delta = self
            .base()
            .upcast_ref::<godot::classes::Node2D>()
            .get_process_delta_time();
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().attack_animation.value -= delta;
        self.move_to(&velocity);

        if time <= 0.0 {
            self.timers().attack_animation.reset();
            self.sm_mut().handle(
                &crate::components::state_machines::enemy_state_machine::EnemyEvent::TimerElapsed,
            );
        }
    }

    fn chain_attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers().chain_attack.value;
        let delta = self.base().upcast_ref().get_process_delta_time();
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().chain_attack.value -= delta;
        self.move_to(&velocity);

        if time <= 0.0 {
            self.timers().chain_attack.reset();
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let time = self.timers().patrol.value;
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity() * speed;
        let delta = self.base().upcast_ref().get_process_delta_time();

        self.update_direction();
        self.move_to(&velocity);
        self.timers().patrol.value -= delta;

        if time <= 0.0 {
            self.timers().patrol.reset();
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let time = self.timers().idle.value;
        let delta = self.base().upcast_ref().get_process_delta_time();
        let velocity = Vector2::ZERO;

        self.move_to(&velocity);
        self.timers().idle.value -= delta;

        if time <= 0.0 {
            self.timers().idle.reset();
            self.set_velocity(
                self.patrol_comp()
                    .get_furthest_distance(self.base().upcast_ref().get_global_position()),
            );
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self
            .base()
            .upcast_ref()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AttackArea");
        let delta = self.base().upcast_ref().get_process_delta_time();
        let speed = self.speeds().aggro;
        let player_position = player.get_position();
        let velocity = Vector2::new(
            self.base()
                .upcast_ref()
                .get_position()
                .direction_to(player_position)
                .x,
            0.0,
        ) * speed;
        self.set_velocity(velocity);
        self.update_direction();
        self.move_to(&velocity);

        if attack_range.has_overlapping_bodies()
            && self.timers().attack_cooldown.value == self.timers().attack_cooldown.initial_value()
        {
            self.timers().attack_cooldown.value -= delta;
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }
}

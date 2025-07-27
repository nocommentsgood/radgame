use godot::builtin::Vector2;
use godot::obj::{Gd, Inherits, WithBaseField};

use crate::classes::characters::main_character::MainCharacter;
use crate::classes::components::speed_component::SpeedComponent;
use crate::classes::components::timer_component::{self as timers, EnemyTimer};
use crate::classes::enemies::patrol_component::PatrolComp;
use crate::components::state_machines::enemy_state_machine;

use super::moveable::{MoveableCharacter, MoveableEntity};

type ET = EnemyTimer;

// TODO: Find a way to conditionally implement traits? Possibly making it generic. Mostly for
// MoveableCharacter versus MoveableEntity

/// Implement on types that use an EnemyStateMachine
/// For types without a base field of CharacterBody2D, see 'EnemyEntityStateMachineExt'.
pub trait EnemyCharacterStateMachineExt: super::has_state::HasState + MoveableCharacter
where
    Self: Inherits<godot::classes::Node2D>
        + WithBaseField<Base: Inherits<godot::classes::CharacterBody2D>>,
{
    fn timers(&mut self) -> &mut timers::Timers;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> SpeedComponent;
    fn patrol_comp(&self) -> &PatrolComp;
    fn get_player_pos(&self) -> Option<Vector2>;

    fn attack(&mut self) {
        let aa = &ET::AttackAnimation;
        let time = self.timers().get(aa);
        let delta = self
            .base()
            .upcast_ref::<godot::classes::Node2D>()
            .get_physics_process_delta_time() as f32;
        let speed = self.speeds().attack;
        let velocity = self.get_velocity();
        self.timers().set(aa, time - delta);
        self.slide(&velocity, &speed);

        if time <= 0.0 {
            self.timers().reset(aa);
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

    fn chain_attack(&mut self) {
        let ac = &ET::AttackChain;
        let time = self.timers().get(ac);
        let delta = self.base().get_physics_process_delta_time() as f32;
        let velocity = self.get_velocity();
        let speed = self.speeds().attack;
        self.timers().set(ac, time - delta);
        self.slide(&velocity, &speed);

        if time <= 0.0 {
            self.timers().reset(ac);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let p = &ET::Patrol;
        let time = self.timers().get(p);
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity();
        let delta = self.base().get_physics_process_delta_time() as f32;

        self.update_direction();
        self.slide(&velocity, &speed);
        self.timers().set(p, time - delta);

        if time <= 0.0 {
            self.timers().reset(p);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let idle = &ET::Idle;
        let time = self.timers().get(idle);
        let delta = self.base().get_physics_process_delta_time() as f32;
        let velocity = Vector2::ZERO;
        self.slide(&velocity, &0.0);
        self.timers().set(idle, time - delta);

        if time <= 0.0 {
            self.timers().reset(idle);
            let v = self
                .patrol_comp()
                .get_furthest_distance(self.base().get_position());
            self.set_velocity(v);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self) {
        let ac = &ET::AttackCooldown;
        let attack_range = self
            .base()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AttackArea");
        let delta = self.base().get_physics_process_delta_time() as f32;
        let speed = self.speeds().aggro;
        if let Some(player_position) = self.get_player_pos() {
            let velocity = Vector2::new(
                self.base().get_position().direction_to(player_position).x,
                0.0,
            );
            self.set_velocity(velocity);
            self.update_direction();
            self.slide(&velocity, &speed);
        }

        if attack_range.has_overlapping_areas()
            && self.timers().get(ac) == self.timers().get_init(ac)
        {
            let time = self.timers().get(ac);
            self.timers().set(ac, time - delta);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }
}

pub trait EnemyEntityStateMachineExt: super::has_state::HasState + MoveableEntity
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node2D>>,
{
    fn timers(&mut self) -> &mut timers::Timers;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> SpeedComponent;
    fn patrol_comp(&self) -> &PatrolComp;
    fn get_player_pos(&self) -> Option<Vector2>;

    fn attack(&mut self) {
        let aa = &ET::AttackAnimation;
        let time = self.timers().get(aa);
        let delta = self
            .base()
            .upcast_ref::<godot::classes::Node2D>()
            .get_process_delta_time() as f32;
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().set(aa, time - delta);
        self.move_to(&velocity, false);

        if time <= 0.0 {
            self.timers().reset(aa);
            self.sm_mut().handle(
                &crate::components::state_machines::enemy_state_machine::EnemyEvent::TimerElapsed,
            );
        }
    }

    fn chain_attack(&mut self) {
        let ac = &ET::AttackChain;
        let time = self.timers().get(ac);
        let delta = self.base().upcast_ref().get_process_delta_time() as f32;
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().set(ac, time - delta);
        self.move_to(&velocity, false);

        if time <= 0.0 {
            self.timers().reset(ac);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let p = &ET::Patrol;
        let time = self.timers().get(p);
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity() * speed;
        let delta = self.base().upcast_ref().get_process_delta_time() as f32;

        self.update_direction();
        self.move_to(&velocity, false);
        self.timers().set(p, time - delta);

        if time <= 0.0 {
            self.timers().reset(p);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let i = &ET::Idle;
        let time = self.timers().get(i);
        let delta = self.base().upcast_ref().get_process_delta_time() as f32;
        let velocity = Vector2::ZERO;

        self.move_to(&velocity, false);
        self.timers().set(i, time - delta);

        if time <= 0.0 {
            self.timers().reset(i);
            let v = self
                .patrol_comp()
                .get_furthest_distance(self.base().upcast_ref().get_global_position());
            self.set_velocity(v);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self) {
        let ac = &ET::AttackCooldown;
        let attack_range = self
            .base()
            .upcast_ref()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AttackArea");
        let delta = self.base().upcast_ref().get_process_delta_time() as f32;
        let time = self.timers().get(ac);
        let speed = self.speeds().aggro;

        if let Some(p) = self.get_player_pos() {
            let velocity = Vector2::new(
                self.base().upcast_ref().get_position().direction_to(p).x,
                0.0,
            ) * speed;
            self.set_velocity(velocity);
            self.update_direction();
            self.move_to(&velocity, false);
        }

        if attack_range.has_overlapping_areas() && time == self.timers().get_init(ac) {
            self.timers().set(ac, time - delta);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }
}

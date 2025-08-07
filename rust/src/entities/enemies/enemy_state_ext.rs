use std::collections::HashMap;
use std::time::Duration;

use bevy_time::Timer;
use godot::{
    builtin::Vector2,
    classes::Node2D,
    obj::{Inherits, WithBaseField},
};

use super::enemy_state_machine::*;
use super::patrol_component::PatrolComp;
use crate::entities::{
    enemies::has_enemy_sensors::HasEnemySensors,
    entity_stats::EntityResources,
    movements::{MoveableCharacter, MoveableEntity, SpeedComponent},
    time::EnemyTimer,
};

type ET = EnemyTimer;

// TODO: Find a way to conditionally implement traits? Possibly making it generic. Mostly for
// MoveableCharacter versus MoveableEntity

/// Implement on types that use an EnemyStateMachine
/// For types without a base field of CharacterBody2D, see 'EnemyEntityStateMachineExt'.
pub trait EnemyCharacterStateMachineExt:
    HasEnemySensors + MoveableCharacter + EntityResources
where
    Self: Inherits<Node2D> + WithBaseField<Base: Inherits<godot::classes::CharacterBody2D>>,
{
    fn timers(&mut self) -> &mut HashMap<EnemyTimer, Timer>;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> &SpeedComponent;
    fn patrol_comp(&self) -> &PatrolComp;
    fn get_player_pos(&self) -> Option<Vector2>;

    fn attack(&mut self) {
        let aa = &ET::AttackAnimation;
        let delta = Duration::from_secs_f32(
            self.base()
                .upcast_ref::<Node2D>()
                .get_physics_process_delta_time() as f32,
        );
        let velocity = self.get_velocity();
        let speed = self.speeds().attack;
        self.timers().get_mut(aa).unwrap().tick(delta);
        self.slide(&velocity, &speed);

        if self.timers().get(aa).unwrap().just_finished() {
            self.timers().get_mut(aa).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn fall(&mut self) {
        let speed = self.speeds().patrol;
        let velocity = Vector2::DOWN;
        self.slide(&velocity, &speed);

        if self.base().is_on_floor() {
            self.sm_mut().handle(&EnemyEvent::OnFloor);
        }
    }

    fn chain_attack(&mut self) {
        let ac = &ET::AttackChainCooldown;
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        let velocity = self.get_velocity();
        let speed = self.speeds().attack;
        self.timers().get_mut(ac).unwrap().tick(delta);
        self.slide(&velocity, &speed);

        if self.timers().get(ac).unwrap().just_finished() {
            self.timers().get_mut(ac).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let p = &ET::Patrol;
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity();
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);

        self.update_direction();
        self.slide(&velocity, &speed);
        self.timers().get_mut(p).unwrap().tick(delta);

        if self.timers().get(p).unwrap().just_finished() {
            self.timers().get_mut(p).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let idle = &ET::Idle;
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        let velocity = Vector2::ZERO;
        self.slide(&velocity, &0.0);
        self.timers().get_mut(idle).unwrap().tick(delta);

        if self.timers().get(idle).unwrap().just_finished() {
            self.timers().get_mut(idle).unwrap().reset();
            let v = self
                .patrol_comp()
                .get_furthest_distance(self.base().get_position());
            self.set_velocity(v);
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    // TODO: Not sure why i was using a timer here...
    fn chase_player(&mut self) {
        // let ac = &ET::AttackChainCooldown;
        // let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
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

        if self.attack_area().has_overlapping_areas()
        // && self.timers().get(ac).unwrap().elapsed_secs() == 0.0
        {
            // self.timers().get_mut(ac).unwrap().tick(delta);
            self.sm_mut().handle(&EnemyEvent::InAttackRange);
        }
    }
}

pub trait EnemyEntityStateMachineExt: HasEnemySensors + MoveableEntity
where
    Self: Inherits<Node2D> + WithBaseField<Base: Inherits<Node2D>>,
{
    fn timers(&mut self) -> &mut HashMap<EnemyTimer, Timer>;
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
    fn speeds(&self) -> &SpeedComponent;
    fn patrol_comp(&self) -> &PatrolComp;
    fn get_player_pos(&self) -> Option<Vector2>;

    fn attack(&mut self) {
        let aa = &ET::AttackAnimation;
        let delta = Duration::from_secs_f32(
            self.base().upcast_ref::<Node2D>().get_process_delta_time() as f32,
        );
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().get_mut(aa).unwrap().tick(delta);
        self.move_to(&velocity, false);

        if self.timers().get(aa).unwrap().just_finished() {
            self.timers().get_mut(aa).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self) {
        let ac = &ET::AttackChainCooldown;
        let delta = Duration::from_secs_f32(
            self.base().upcast_ref::<Node2D>().get_process_delta_time() as f32,
        );
        let speed = self.speeds().attack;
        let velocity = self.get_velocity() * speed;
        self.timers().get_mut(ac).unwrap().tick(delta);
        self.move_to(&velocity, false);

        if self.timers().get(ac).unwrap().just_finished() {
            self.timers().get_mut(ac).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn patrol(&mut self) {
        let p = &ET::Patrol;
        let speed = self.speeds().patrol;
        let velocity = self.get_velocity() * speed;
        let delta = Duration::from_secs_f32(
            self.base().upcast_ref::<Node2D>().get_process_delta_time() as f32,
        );

        self.update_direction();
        self.move_to(&velocity, false);
        self.timers().get_mut(p).unwrap().tick(delta);

        if self.timers().get(p).unwrap().just_finished() {
            self.timers().get_mut(p).unwrap().reset();
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        let idle = &ET::Idle;
        let velocity = Vector2::ZERO;
        let delta = Duration::from_secs_f32(
            self.base().upcast_ref::<Node2D>().get_process_delta_time() as f32,
        );

        self.timers().get_mut(idle).unwrap().tick(delta);
        self.move_to(&velocity, false);

        if self.timers().get(idle).unwrap().just_finished() {
            self.timers().get_mut(idle).unwrap().reset();
            let v = self
                .patrol_comp()
                .get_furthest_distance(self.base().upcast_ref::<Node2D>().get_position());
            self.set_velocity(v);
            self.sm_mut().handle(&EnemyEvent::TimerElapsed);
        }
    }

    // TODO: Not sure why i was using a timer here...
    fn chase_player(&mut self) {
        // let ac = &ET::AttackChainCooldown;
        // let delta = self.base().upcast_ref::<Node2D>().get_process_delta_time() as f32;
        let speed = self.speeds().aggro;

        if let Some(p) = self.get_player_pos() {
            let velocity = Vector2::new(
                self.base()
                    .upcast_ref::<Node2D>()
                    .get_position()
                    .direction_to(p)
                    .x,
                0.0,
            ) * speed;
            self.set_velocity(velocity);
            self.update_direction();
            self.move_to(&velocity, false);
        }

        if self.attack_area().has_overlapping_areas()
        // && self.timers().get(ac).unwrap().elapsed_secs() == 0.0
        {
            // self.timers().get_mut(ac).unwrap().tick(delta);
            self.sm_mut().handle(&EnemyEvent::InAttackRange);
        }
    }
}

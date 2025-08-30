use std::collections::HashMap;

use godot::{
    builtin::Vector2,
    classes::{Area2D, Node2D, Timer},
    meta::FromGodot,
    obj::{DynGd, Gd, Inherits, WithBaseField, WithUserSignals},
};

use super::{enemy_state_machine::*, patrol_component::PatrolComp};
use crate::entities::{
    damage::{Damageable, Damaging},
    enemies::{animatable::Animatable, has_enemy_sensors::HasEnemySensors},
    movements::SpeedComponent,
    player::main_character::MainCharacter,
    time::EnemyTimer,
};

type ET = EnemyTimer;

// TODO: Figure out how to register functions as signals in traits.
/// The idea behind this was to provide a trait that could be used to implement basic enemy
/// functionality: movement, sending events to the state machine, etc. When a new enemy class is created, almost
/// all functionality would be provided by implementing `EnemyEntityStateMachineExt`.
//
// In practice, I don't think it is modular enough, and should probably be replaced with some sort
// of composition. I think that would also remove these weird trait bounds.
pub trait EnemyEntityStateMachineExt:
    HasEnemySensors + crate::entities::movements::Move + Animatable
where
    Self: Inherits<Node2D> + WithBaseField<Base: Inherits<Node2D>> + WithUserSignals,
{
    fn timers(&mut self) -> &mut HashMap<EnemyTimer, Gd<Timer>>;
    fn speeds(&self) -> &SpeedComponent;
    fn patrol_comp(&self) -> &PatrolComp;
    fn get_player_pos(&self) -> Option<Vector2>;
    fn get_chain_attack_count(&self) -> u32;
    fn set_chain_attack_count(&mut self, amount: u32);

    /// The method to call during attacking states.
    /// For example, this is where the `ProjectileEnemy` spawns it's projectile.
    fn attack_implementation(&mut self);

    fn connect_signals(&mut self) {
        let this = self.to_gd();
        self.timers()
            .get_mut(&ET::AttackAnimation)
            .unwrap()
            .signals()
            .timeout()
            .connect_other(&this, Self::on_attack_animation_timeout);
        self.timers()
            .get_mut(&ET::AttackChainCooldown)
            .unwrap()
            .signals()
            .timeout()
            .connect_other(&this, Self::on_chain_attack_timeout);
        self.timers()
            .get_mut(&ET::Patrol)
            .unwrap()
            .signals()
            .timeout()
            .connect_other(&this, Self::on_patrol_timeout);
        self.timers()
            .get_mut(&ET::Idle)
            .unwrap()
            .signals()
            .timeout()
            .connect_other(&this, Self::on_idle_timeout);
        self.aggro_area_mut()
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_aggro_area_entered);
        self.aggro_area_mut()
            .signals()
            .area_exited()
            .connect_other(&this, Self::on_aggro_area_exited);
        self.hitbox_mut()
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hitbox);
    }

    fn fall(&mut self) {
        let velocity = Vector2::DOWN * self.speeds().aggro;
        self.set_velocity(velocity);
        self.slide();
    }

    // TODO: Change this when typed attacks are finished.
    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        damaging.dyn_bind().do_damage(damageable);
    }

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player")
            && let Some(player) = area.get_parent()
            && let Ok(player) = player.try_cast::<MainCharacter>()
        {
            let speed = self.speeds().aggro;
            self.set_player_pos(Some(player.get_global_position()));
            let velocity = Vector2::new(
                self.base()
                    .upcast_ref::<Node2D>()
                    .get_position()
                    .direction_to(player.get_global_position())
                    .x,
                0.0,
            ) * speed;
            self.set_velocity(velocity);
            self.update_animation();
            self.transition_sm(&EnemyEvent::FoundPlayer);
        }
    }

    fn track_player(&mut self) {
        let areas = self.aggro_area().get_overlapping_areas();
        for area in areas.iter_shared() {
            if area.is_in_group("player") {
                let player = area.get_parent().unwrap().cast::<MainCharacter>();
                self.set_player_pos(Some(player.get_global_position()));
            }
        }
    }

    fn on_aggro_area_exited(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            self.set_player_pos(None);
            self.transition_sm(&EnemyEvent::LostPlayer);
        }
    }

    fn attack(&mut self) {
        self.timers().get_mut(&ET::AttackAnimation).unwrap().start();
        self.timers().get_mut(&ET::AttackCooldown).unwrap().start();
        self.track_player();
        self.attack_implementation();
    }

    fn on_attack_animation_timeout(&mut self) {
        self.transition_sm(&EnemyEvent::TimerElapsed);
    }

    fn chain_attack(&mut self) {
        self.attack_implementation();
        self.timers()
            .get_mut(&ET::AttackChainCooldown)
            .unwrap()
            .start();
    }

    fn on_chain_attack_timeout(&mut self) {
        if self.get_chain_attack_count() >= 2 {
            self.set_chain_attack_count(0);
            self.timers().get_mut(&ET::AttackCooldown).unwrap().start();
            self.transition_sm(&EnemyEvent::TimerElapsed);
        } else {
            self.set_chain_attack_count(self.get_chain_attack_count() + 1);
            self.chain_attack();
        }
    }

    /// Run in `process()`
    fn patrol(&mut self) {
        if self.timers().get(&ET::Patrol).unwrap().is_stopped() {
            self.timers().get_mut(&ET::Patrol).unwrap().start();
        }

        self.slide();
    }

    fn on_patrol_timeout(&mut self) {
        self.transition_sm(&EnemyEvent::TimerElapsed);
    }

    fn idle(&mut self) {
        self.timers().get_mut(&ET::Idle).unwrap().start();
    }

    fn on_idle_timeout(&mut self) {
        let v = self
            .patrol_comp()
            .get_furthest_distance(self.base().upcast_ref::<Node2D>().get_position());
        self.set_velocity(v * self.speeds().aggro);
        self.transition_sm(&EnemyEvent::TimerElapsed);
    }

    /// Run in `process()`
    fn chase_player(&mut self) {
        let speed = self.speeds().aggro;
        self.track_player();

        if let Some(p) = self.get_player_pos() {
            let velocity = Vector2::new(
                self.base()
                    .upcast_ref::<Node2D>()
                    .get_position()
                    .direction_to(p)
                    .x,
                0.0,
            );
            self.set_velocity(velocity.normalized_or_zero() * speed);
            self.slide();
        }

        if self.attack_area().has_overlapping_areas()
            && self.timers()[&ET::AttackCooldown].get_time_left() == 0.0
        {
            self.transition_sm(&EnemyEvent::InAttackRange);
        }
    }

    fn transition_sm(&mut self, event: &EnemyEvent) {
        let cur = self.sm().state().as_discriminant();
        self.sm_mut().handle(event);
        let new = self.sm().state().as_discriminant();

        if cur != new {
            self.update_animation();
            self.on_sm_state_changed();
        }
    }

    fn on_sm_state_changed(&mut self) {
        match self.sm().state() {
            State::Attack {} => self.attack(),
            State::Attack2 {} => self.chain_attack(),
            State::Idle {} => self.idle(),
            State::Patrol {} => self.patrol(),
            State::Falling {} => self.fall(),
            State::ChasePlayer {} => self.chase_player(),
        }
    }
}

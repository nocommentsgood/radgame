use godot::{
    classes::{Area2D, IArea2D},
    prelude::*,
};

use crate::{
    classes::{
        characters::entity_hitbox::EntityHitbox, scenes::triggerable_objects::TriggerableEnvObject,
    },
    utils::collision_layers::CollisionLayers,
};

/// An Area2D that, when triggered by the player entering it, will activate all of the triggerable objects it
/// is connected to.
#[derive(GodotClass)]
#[class(base = Area2D, init)]
pub struct EnvironmentTrigger {
    #[export]
    triggerable_objects: Array<DynGd<Node2D, dyn TriggerableEnvObject>>,

    // I will not give up Rust enums...
    /// 1 = OneShot, 2 = Persistent, 3 = Limited
    ///
    #[export]
    trigger_type_hint: u32,

    /// If the type hint is set to Limited, this is the number of times the trigger can be
    /// activated.
    #[export]
    trigger_times: u32,

    times_triggered: u32,

    pub trigger_ty: Option<TriggerType>,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for EnvironmentTrigger {
    fn ready(&mut self) {
        self.trigger_ty = match self.trigger_type_hint {
            1 => Some(TriggerType::OneShot),
            2 => Some(TriggerType::Persistent),
            3 => {
                if self.trigger_times == 0 {
                    godot_warn!(
                        "EnvironmentTrigger is set to Limited but trigger_times is 0. Defaulting to OneShot."
                    );
                    Some(TriggerType::OneShot)
                } else {
                    Some(TriggerType::Limited(self.trigger_times))
                }
            }
            _ => None,
        };

        self.base_mut()
            .set_collision_layer_value(CollisionLayers::WorldEffects as i32, true);
        self.base_mut()
            .set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        self.signals()
            .area_entered()
            .connect_self(Self::on_player_enters_trigger);
    }
}

#[godot_api]
impl EnvironmentTrigger {
    #[signal]
    pub fn trigger_activated(trigger: Gd<EnvironmentTrigger>);

    fn on_player_enters_trigger(&mut self, area: Gd<Area2D>) {
        if let Ok(h_box) = area.try_cast::<EntityHitbox>()
            && let Some(_player) = h_box.get_owner()
        {
            for mut i in self.triggerable_objects.iter_shared() {
                i.dyn_bind_mut().on_activated();
            }
        }

        if let Some(t) = &self.trigger_ty {
            match t {
                TriggerType::OneShot => {
                    self.base_mut().queue_free();
                }
                TriggerType::Limited(num) => {
                    if self.times_triggered >= *num {
                        self.base_mut().queue_free();
                    } else {
                        self.times_triggered += 1;
                    }
                }
                TriggerType::Persistent => (),
            }
        } else {
            panic!("Trigger type must be set.");
        }
    }
}

pub enum TriggerType {
    OneShot,
    Persistent,
    Limited(u32),
}

use godot::{
    classes::{Area2D, CollisionShape2D, IArea2D, IStaticBody2D, StaticBody2D},
    prelude::*,
};

use crate::{
    classes::characters::{entity_hitbox::EntityHitbox, main_character::MainCharacter},
    utils::collision_layers::CollisionLayers,
};

pub trait TriggerableEnvObject {
    fn on_activated(&mut self);
}

#[derive(Copy, Clone)]
pub enum TriggerDuration {
    OneShot,
    Persistent,
    Limited(u32),
}

/// An Area2D that, when triggered by the player entering it, will activate all of the triggerable objects it
/// is connected to.
#[derive(GodotClass)]
#[class(base = Area2D, init)]
pub struct EnvironmentTrigger {
    #[export]
    triggerable_objects: Array<DynGd<Node, dyn TriggerableEnvObject>>,

    // Work around Godot not supporting discriminated unions.
    /// 1 = OneShot, 2 = Persistent, 3 = Limited
    #[export]
    trigger_type_hint: u32,

    /// If the type hint is set to Limited, this is the number of times the trigger can be
    /// activated.
    #[export]
    trigger_times: u32,

    pub trigger_ty: Option<TriggerDuration>,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for EnvironmentTrigger {
    fn ready(&mut self) {
        // Get trigger type from the editor.
        self.trigger_ty = match self.trigger_type_hint {
            1 => Some(TriggerDuration::OneShot),
            2 => Some(TriggerDuration::Persistent),
            3 => {
                if self.trigger_times <= 1 {
                    godot_warn!(
                        "EnvironmentTrigger is set to Limited but trigger_times is <= 1. Defaulting to OneShot."
                    );
                    Some(TriggerDuration::OneShot)
                } else {
                    Some(TriggerDuration::Limited(self.trigger_times))
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
    fn on_player_enters_trigger(&mut self, area: Gd<Area2D>) {
        // TODO: This check of getting the player is used in a few other places too. Maybe it
        // should be exposed from a singleton.
        if let Ok(h_box) = area.try_cast::<EntityHitbox>()
            && let Some(player) = h_box.get_owner()
            && let Ok(_player) = player.try_cast::<MainCharacter>()
        {
            // BUG: If the editor has an empty element, this will panic. Not sure how to guard
            // against this as the type is Variant.
            for mut i in self.triggerable_objects.iter_shared() {
                i.dyn_bind_mut().on_activated();
            }
        }

        if let Some(t) = self.trigger_ty {
            match t {
                TriggerDuration::OneShot => {
                    self.base_mut().queue_free();
                }
                TriggerDuration::Limited(mut num) => {
                    num -= 1;
                    if num > 0 {
                        self.trigger_ty.replace(TriggerDuration::Limited(num));
                    } else {
                        self.base_mut().queue_free();
                    }
                }
                TriggerDuration::Persistent => (),
            }
        }
    }
}

#[derive(GodotClass)]
#[class(init, base = StaticBody2D)]
pub struct ClosingDoor {
    /// The final open position of the door.
    #[export(range = (0.0, -1.0, or_less))]
    open_position: Vector2,

    /// The final closed position of the door.
    #[export(range = (0.0, 1.0, or_greater))]
    closed_position: Vector2,

    is_closed: bool,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for ClosingDoor {
    fn ready(&mut self) {
        self.is_closed = false;
        self.base_mut().set_process(false);
        let mut shape = self
            .base()
            .get_node_as::<CollisionShape2D>("CollisionShape2D");
        shape.set_disabled(true);
    }

    fn process(&mut self, delta: f32) {
        if !self.is_closed {
            if self.base().get_position().y > self.get_closed_position().y {
                let position = self.base().get_position();
                let x = self.get_closed_position().x;
                self.base_mut()
                    .set_position(Vector2::new(x, position.y + Vector2::UP.y * 20.0 * delta));
            } else {
                self.is_closed = true;
                self.base_mut().set_process(false);
            }
        }

        if self.is_closed {
            if self.base().get_position().y < self.get_open_position().y {
                let position = self.base().get_position();
                let x = self.get_open_position().x;
                self.base_mut()
                    .set_position(Vector2::new(x, position.y + Vector2::DOWN.y * 20.0 * delta));
            } else {
                self.is_closed = false;
                self.base_mut().set_process(false);
            }
        }
    }
}

#[godot_dyn]
impl TriggerableEnvObject for ClosingDoor {
    fn on_activated(&mut self) {
        let mut shape = self
            .base()
            .get_node_as::<CollisionShape2D>("CollisionShape2D");
        if self.is_closed {
            shape.apply_deferred(|this| this.set_disabled(true));
            self.base_mut().set_process(true);
        } else {
            shape.apply_deferred(|this| this.set_disabled(false));
            self.base_mut().set_process(true);
        }
    }
}

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct MapTransition {
    #[export]
    next_map: OnEditor<Gd<PackedScene>>,
    base: Base<Node>,
}

#[godot_api]
impl MapTransition {
    #[signal]
    pub fn transition_maps(next_map: Gd<PackedScene>);
}

#[godot_dyn]
impl TriggerableEnvObject for MapTransition {
    fn on_activated(&mut self) {
        let next = self.next_map.clone();
        self.signals().transition_maps().emit(&next);
    }
}

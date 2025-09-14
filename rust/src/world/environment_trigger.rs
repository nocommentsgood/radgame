use crate::{
    entities::{
        entity_hitbox::EntityHitbox,
        player::{
            main_character::MainCharacter,
            shaky_player_camera::{CameraFollowType, PlayerCamera, TweenEase, TweenTransition},
        },
    },
    utils::collision_layers::CollisionLayers,
};
use godot::{
    classes::{
        Area2D, Camera2D, CollisionShape2D, IArea2D, IStaticBody2D, Marker2D, StaticBody2D,
        tween::TransitionType,
    },
    prelude::*,
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
        // if let Ok(h_box) = area.try_cast::<EntityHitbox>()
        //     && let Some(player) = h_box.get_owner()
        //     && let Ok(_player) = player.try_cast::<MainCharacter>()
        // {
        // BUG: If the editor has an empty element, this will panic. Not sure how to guard
        // against this as the type is Variant.
        for mut i in self.triggerable_objects.iter_shared() {
            i.dyn_bind_mut().on_activated();
            // }
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
    #[init(sentinel = StringName::from(c""))]
    next_map_scene: OnEditor<StringName>,

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
        let next = load(self.next_map_scene.arg());
        self.signals().transition_maps().emit(&next);
    }
}

#[derive(GodotClass)]
#[class(init, base = Marker2D)]
pub struct SceneTransition {
    base: Base<Marker2D>,
}

#[godot_api]
impl SceneTransition {
    #[signal]
    pub fn scene_transition(position: Gd<Marker2D>);
}

#[godot_dyn]
impl TriggerableEnvObject for SceneTransition {
    fn on_activated(&mut self) {
        println!("Emitting scene trans signal");
        let marker = self.to_gd().upcast();
        self.signals().scene_transition().emit(&marker);
    }
}

#[derive(GodotClass)]
#[class(init, base = Node2D)]
pub struct CameraData {
    // Initialized by the `Main` node.
    pub player_camera: Option<Gd<PlayerCamera>>,

    #[export]
    next_pos: Vector2,

    #[export]
    pub zoom: Vector2,

    #[export]
    #[export_group(name = "Tweening")]
    duration: f32,
    #[export]
    #[export_group(name = "Tweening")]
    pub transition: TweenTransition,

    #[export]
    #[export_group(name = "Tweening")]
    pub ease: TweenEase,

    #[export]
    #[export_group(name = "Follow")]
    follow: CameraFollowType,

    base: Base<Node2D>,
}

#[godot_dyn]
impl TriggerableEnvObject for CameraData {
    fn on_activated(&mut self) {
        let mut zoom_tween = self.base_mut().create_tween().unwrap();
        zoom_tween.set_trans(self.transition.to_type());
        zoom_tween.set_ease(self.ease.to_type());
        zoom_tween.tween_property(
            self.player_camera.as_ref().unwrap(),
            "zoom",
            &self.zoom.to_variant(),
            self.duration.into(),
        );

        match self.follow {
            CameraFollowType::Simple => {
                // if let Ok(mut player) = self
                //     .player_camera
                //     .as_ref()
                //     .unwrap()
                //     .get_parent()
                //     .unwrap()
                //     .try_cast::<MainCharacter>()
                if self.player_camera.as_ref().unwrap().get_parent().is_none() {
                    // if player
                    //     .try_get_node_as::<PlayerCamera>("ShakyPlayerCamera")
                    //     .is_none()
                    // {
                    // player.add_child(self.player_camera.as_ref().unwrap());
                    self.player_camera
                        .as_mut()
                        .unwrap()
                        .signals()
                        .request_attach()
                        .emit();
                    println!("Emitted attach request");
                    //                     }
                }
            }
            CameraFollowType::Frame => {
                if let Ok(mut player) = self
                    .player_camera
                    .as_ref()
                    .unwrap()
                    .get_parent()
                    .unwrap()
                    .try_cast::<MainCharacter>()
                {
                    if player
                        .try_get_node_as::<PlayerCamera>("ShakyPlayerCamera")
                        .is_none()
                    {
                        player.add_child(self.player_camera.as_ref().unwrap());
                    }
                }
            }
            CameraFollowType::None => {
                println!("Emitting cam sig becasue Non");
                println!("Data position: {}", self.base().get_global_position());
                let pos = self
                    .player_camera
                    .as_ref()
                    .unwrap()
                    .get_screen_center_position();
                self.player_camera.as_mut().unwrap().set_enabled(false);
                let mut temp_cam = Camera2D::new_alloc();
                temp_cam.set_enabled(true);
                temp_cam.set_position(pos);
                self.base_mut().add_child(&temp_cam)
                // self.player_camera
                //     .as_ref()
                //     .unwrap()
                //     .signals()
                //     .request_detach()
                //     .emit(pos);
            }
        }
    }
}

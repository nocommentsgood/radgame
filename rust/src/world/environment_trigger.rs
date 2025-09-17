use crate::{
    entities::player::shaky_player_camera::PlayerCamera, utils::collision_layers::CollisionLayers,
};
use godot::{
    classes::{
        Area2D, CollisionShape2D, IArea2D, IStaticBody2D, Marker2D, StaticBody2D, Tween,
        tween::TransitionType,
    },
    obj::{EngineEnum, WithBaseField},
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
        // BUG: If the editor has an empty element, this will panic. Not sure how to guard
        // against this as the type is Variant.
        for mut i in self.triggerable_objects.iter_shared() {
            i.dyn_bind_mut().on_activated();
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
        let marker = self.to_gd().upcast();
        self.signals().scene_transition().emit(&marker);
    }
}

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct CameraData {
    // Initialized by the `Main` node.
    pub player_camera: Option<Gd<PlayerCamera>>,

    #[export]
    shape: OnEditor<Gd<CollisionShape2D>>,

    #[export]
    #[export_group(name = "CameraProps")]
    pub zoom: Vector2,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = i32::MIN)]
    left: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = i32::MIN)]
    top: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = i32::MAX)]
    right: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = i32::MAX)]
    bottom: i32,

    #[export]
    #[export_subgroup(name = "Drag")]
    horiztonal_enabled: bool,

    #[export]
    #[export_subgroup(name = "Drag")]
    vertical_enabled: bool,

    #[export(range = (-1.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    horizontal_offset: f32,

    #[export(range = (-1.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    vertical_offset: f32,

    #[export(range = (0.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    left_margin: f32,

    #[export(range = (0.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    top_margin: f32,

    #[export(range = (0.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    right_margin: f32,

    #[export(range = (0.0, 1.0))]
    #[export_subgroup(name = "Drag")]
    bottom_margin: f32,

    prev_horizontal_enabled: Option<bool>,
    prev_vertical_enabled: Option<bool>,
    prev_zoom: Option<Vector2>,
    prev_offset: Option<Vector2>,
    prev_drag_margin: Vec<Option<(Side, f32)>>,
    prev_limits: Vec<Option<(Side, i32)>>,

    base: Base<Area2D>,
}

#[godot_api]
impl CameraData {
    fn apply_camera_properties(&mut self) {
        if let Some(camera) = self.player_camera.as_mut() {
            // Obtain current camera properties.
            self.prev_horizontal_enabled = Some(camera.is_drag_horizontal_enabled());
            self.prev_vertical_enabled = Some(camera.is_drag_vertical_enabled());
            self.prev_zoom = Some(camera.get_zoom());
            self.prev_offset = Some(camera.get_offset());
            for s in Side::values() {
                self.prev_drag_margin
                    .push(Some((*s, camera.get_drag_margin(*s))));
                self.prev_limits.push(Some((*s, camera.get_limit(*s))));
            }

            camera.set_zoom(self.zoom);

            // Apply new camera properties.
            camera.set_limit(Side::LEFT, self.left);
            camera.set_limit(Side::TOP, self.top);
            camera.set_limit(Side::RIGHT, self.right);
            camera.set_limit(Side::BOTTOM, self.bottom);

            camera.set_drag_horizontal_enabled(self.horiztonal_enabled);
            camera.set_drag_vertical_enabled(self.vertical_enabled);
            // if camera.is_drag_horizontal_enabled() {
            //     camera.bind_mut().manual_drag = false;
            //     camera.bind_mut().reset_x_offset();
            // } else {
            //     camera.bind_mut().manual_drag = true;
            // }

            camera.set_drag_margin(Side::LEFT, self.left_margin);
            camera.set_drag_margin(Side::TOP, self.top_margin);
            camera.set_drag_margin(Side::RIGHT, self.right_margin);
            camera.set_drag_margin(Side::BOTTOM, self.bottom_margin);
        }
    }

    fn reset_camera_properties(&mut self) {
        if let Some(camera) = self.player_camera.as_mut() {
            camera.set_drag_horizontal_enabled(self.prev_horizontal_enabled.unwrap());
            camera.set_drag_vertical_enabled(self.prev_vertical_enabled.unwrap());
            camera.set_zoom(self.prev_zoom.unwrap());
            // camera.set_offset(self.prev_offset.unwrap());

            for (side, val) in self.prev_limits.iter().flatten() {
                camera.set_limit(*side, *val);
            }
            for (side, val) in self.prev_drag_margin.iter().flatten() {
                camera.set_drag_margin(*side, *val);
            }
        }
    }

    fn on_player_entered(&mut self, area: Gd<Area2D>) {
        println!("Player entered");
        self.apply_camera_properties();
    }

    fn on_player_exit(&mut self, area: Gd<Area2D>) {
        self.reset_camera_properties();
    }
}

#[godot_api]
impl IArea2D for CameraData {
    fn ready(&mut self) {
        self.signals()
            .area_entered()
            .connect_self(Self::on_player_entered);

        self.signals()
            .area_exited()
            .connect_self(Self::on_player_exit);
    }
}

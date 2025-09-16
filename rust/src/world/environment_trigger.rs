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
        Area2D, Camera2D, CollisionShape2D, IArea2D, ICamera2D, IStaticBody2D, Marker2D,
        StaticBody2D, Tween, tween::TransitionType,
    },
    obj::WithBaseField,
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
    #[export_group(name = "CameraProps")]
    next_pos: Vector2,

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
    #[export]
    #[export_subgroup(name = "Drag")]
    left_margin: f32,
    #[export]
    #[export_subgroup(name = "Drag")]
    top_margin: f32,
    #[export]
    #[export_subgroup(name = "Drag")]
    right_margin: f32,
    #[export]
    #[export_subgroup(name = "Drag")]
    bottom_margin: f32,

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

#[godot_api]
impl CameraData {
    /// Removes the camera from the player, forcing the camera to stop following. Handled by the
    /// `Main` node.
    #[signal]
    pub fn detach_camera();

    /// Adds the camera as a child of the player, forcing the camera to follow. Handled by the
    /// `Main` node.
    #[signal]
    pub fn attach_camera();

    fn apply_camera_properties(&mut self) {
        if let Some(camera) = self.player_camera.as_mut() {
            camera.set_limit(Side::LEFT, self.left);
            camera.set_limit(Side::TOP, self.top);
            camera.set_limit(Side::RIGHT, self.right);
            camera.set_limit(Side::BOTTOM, self.bottom);

            camera.set_drag_horizontal_enabled(self.horiztonal_enabled);
            camera.set_drag_vertical_enabled(self.vertical_enabled);
            if camera.is_drag_horizontal_enabled() {
                camera.bind_mut().manual_drag = false;
                camera.bind_mut().reset_x_offset();
            } else {
                camera.bind_mut().manual_drag = true;
            }

            camera.set_drag_margin(Side::LEFT, self.left_margin);
            camera.set_drag_margin(Side::TOP, self.top_margin);
            camera.set_drag_margin(Side::RIGHT, self.right_margin);
            camera.set_drag_margin(Side::BOTTOM, self.bottom_margin);
        }
    }
}

#[godot_dyn]
impl TriggerableEnvObject for CameraData {
    fn on_activated(&mut self) {
        match self.follow {
            CameraFollowType::Tight => {
                if let Some(p_cam) = self.player_camera.as_ref()
                    && !p_cam.bind().is_following
                {
                    self.signals().attach_camera().emit();
                }

                self.apply_camera_properties();

                if let Some(p_cam) = self.player_camera.as_mut() {
                    p_cam.bind_mut().manual_drag = false;
                    p_cam.bind_mut().reset_x_offset();
                    if p_cam.is_drag_horizontal_enabled() | p_cam.is_drag_vertical_enabled() {
                        godot_warn!(
                            "FollowType set to Tight but drag is enabled. Change FollowType variant or disable drag. Defaulting to disabled"
                        );
                        p_cam.set_drag_vertical_enabled(false);
                        p_cam.set_drag_horizontal_enabled(false);
                        p_cam.set_position_smoothing_enabled(false);
                    }
                }
                if self.player_camera.as_ref().unwrap().get_zoom() != self.zoom {
                    let mut tween = self.base_mut().create_tween().unwrap();
                    tween.set_trans(self.transition.to_type());
                    tween.set_ease(self.ease.to_type());
                    tween.tween_property(
                        self.player_camera.as_ref().unwrap(),
                        "zoom",
                        &self.zoom.to_variant(),
                        self.duration.into(),
                    );
                }
            }
            CameraFollowType::Simple => {
                if let Some(p_cam) = self.player_camera.as_ref()
                    && !p_cam.bind().is_following
                {
                    self.signals().attach_camera().emit();
                }

                self.apply_camera_properties();
                self.player_camera.as_mut().unwrap().bind_mut().manual_drag = true;
                self.player_camera
                    .as_mut()
                    .unwrap()
                    .set_position_smoothing_enabled(true);
                if self.player_camera.as_ref().unwrap().get_zoom() != self.zoom {
                    let mut tween = self.base_mut().create_tween().unwrap();
                    tween.set_trans(self.transition.to_type());
                    tween.set_ease(self.ease.to_type());
                    tween.tween_property(
                        self.player_camera.as_ref().unwrap(),
                        "zoom",
                        &self.zoom.to_variant(),
                        self.duration.into(),
                    );
                }
            }
            CameraFollowType::Frame => {
                if let Some(p_cam) = &self.player_camera
                    && !p_cam.bind().is_following
                {
                    self.signals().attach_camera().emit();
                }
                self.apply_camera_properties();

                if self.player_camera.as_ref().unwrap().get_zoom() != self.zoom {
                    let mut tween = self.base_mut().create_tween().unwrap();
                    tween.set_trans(self.transition.to_type());
                    tween.set_ease(self.ease.to_type());
                    tween.tween_property(
                        self.player_camera.as_ref().unwrap(),
                        "zoom",
                        &self.zoom.to_variant(),
                        self.duration.into(),
                    );
                }
            }
            CameraFollowType::None => {
                if self.player_camera.as_ref().unwrap().bind().is_following {
                    self.signals().detach_camera().emit();
                }

                self.apply_camera_properties();

                if let Some(p_cam) = self.player_camera.as_mut() {
                    p_cam.bind_mut().manual_drag = false;
                    p_cam.set_drag_vertical_enabled(false);
                    p_cam.set_drag_horizontal_enabled(false);
                }

                if self.player_camera.as_ref().unwrap().get_zoom() != self.zoom {
                    let mut tween = self.base_mut().create_tween().unwrap();
                    tween.set_trans(self.transition.to_type());
                    tween.set_ease(self.ease.to_type());
                    tween.tween_property(
                        self.player_camera.as_ref().unwrap(),
                        "zoom",
                        &self.zoom.to_variant(),
                        self.duration.into(),
                    );
                }
            }
        }
    }
}

#[derive(GodotClass)]
#[class(base = Camera2D, init)]
pub struct NewTestCamera {
    #[init(node = "Area2D")]
    area: OnReady<Gd<Area2D>>,
    prev_pos: Option<Vector2>,
    prev_zoom: Option<Vector2>,
    #[init(val = Box::new(MoveZoomCameraEffect))]
    effect: Box<dyn CameraEffect>,
    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for NewTestCamera {
    fn ready(&mut self) {
        self.base_mut().set_physics_process(false);
        self.base_mut().set_process(false);
        self.area
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_player_entered);

        self.area
            .signals()
            .area_exited()
            .connect_other(&self.to_gd(), Self::on_player_exit);
    }

    fn physics_process(&mut self, delta: f32) {
        println!("Zooming");
        MoveZoomCameraEffect::zoom(
            self,
            Vector2::new(1.2, 1.2),
            Vector2::new(0.9, 0.9),
            Vector2::new(1887.0, 35.0),
            delta,
        );
    }
}

#[godot_api]
impl NewTestCamera {
    fn on_player_entered(&mut self, area: Gd<Area2D>) {
        let mut player = area.get_node_as::<MainCharacter>("..");
        player.bind_mut().camera.set_enabled(false);
        self.prev_pos = Some(player.bind().camera.get_global_position());
        self.prev_zoom = Some(player.bind().camera.get_zoom());
        self.apply_deferred(move |this| {
            this.base_mut().set_physics_process(true);
            this.base_mut().set_enabled(true);
            this.base_mut().make_current();
            this.base_mut()
                .set_global_position(player.bind().camera.get_global_position());
            // let mut tween = self.base().create_tween().unwrap();
            // tween.tween_property(&self, "zoom", );
        });
    }

    fn on_player_exit(&mut self, area: Gd<Area2D>) {
        let mut player = area.get_node_as::<MainCharacter>("..");
        self.base_mut().set_enabled(false);
        player.bind_mut().camera.set_enabled(true);
        player.bind_mut().camera.make_current();
        self.prev_pos = None;
        self.prev_zoom = None;
    }
}

struct MoveZoomCameraEffect;

trait CameraEffect {}

impl MoveZoomCameraEffect {
    fn zoom(
        camera: &mut NewTestCamera,
        max_zoom: Vector2,
        min_zoom: Vector2,
        target_position: Vector2,
        delta: f32,
    ) {
        let area = &*camera.area;
        let areas = area.get_overlapping_areas();
        for area in areas.iter_shared() {
            if let Ok(h_box) = area.try_cast::<EntityHitbox>() {
                let player = h_box.get_node_as::<MainCharacter>("..");
                if player.bind().velocity.x > 0.0 {
                    let cur_zoom = camera.base().get_zoom();
                    let cur_pos = camera.base().get_global_position();
                    let new_pos = cur_pos.move_toward(target_position, 50.0 * delta);
                    let target = cur_zoom.lerp(max_zoom, delta * 1.0);
                    camera.base_mut().set_zoom(target);
                    camera.base_mut().set_global_position(new_pos);
                } else if player.bind().velocity.x < 0.0 {
                    let z = camera.base().get_zoom();
                    let cur_pos = camera.base().get_global_position();
                    let new_pos = cur_pos.move_toward(-target_position, 50.0 * delta);
                    let target = z.lerp(min_zoom, delta * 1.0);
                    camera.base_mut().set_zoom(target);
                    camera.base_mut().set_global_position(new_pos);
                }
            }
        }
    }
}

impl CameraEffect for MoveZoomCameraEffect {}

use fastnoise_lite::FastNoiseLite;
use godot::{
    classes::{Area2D, Camera2D, CollisionShape2D, IArea2D, ICamera2D},
    prelude::*,
};

#[allow(dead_code)]
pub enum TraumaLevel {
    Low,
    Med,
    High,
}

impl From<i64> for TraumaLevel {
    fn from(value: i64) -> Self {
        match value {
            30..=i64::MAX => Self::High,
            10..=29 => Self::Med,
            _ => Self::Low,
        }
    }
}

// BUG: Offset doesn't seem to reset to original position after trauma is gone.
#[derive(GodotClass)]
#[class(base = Camera2D, init)]
pub struct PlayerCamera {
    // TODO: Finalize and remove exports.
    #[export]
    decay: f32,
    max_offset: Vector2,
    #[export]
    max_rot: f32,
    #[init(val = FastNoiseLite::new())]
    noise: FastNoiseLite,
    original_offset: Vector2,
    trauma: f32,
    noise_y: i32,
    set_right: Option<bool>,
    #[init(val = true)]
    pub enable_manual_drag: bool,
    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for PlayerCamera {
    fn physics_process(&mut self, delta: f32) {
        if self.trauma > 0.0 {
            self.trauma = f32::max(self.trauma - self.decay * delta, 0.0);
            self.rust_shake();
        }

        // Lerp towards the players last movement.
        if let Some(b) = self.set_right
            && self.enable_manual_drag
        {
            let cur_pos = self.base().get_offset();
            let target = if b {
                Vector2::new(50.0, self.base().get_offset().y)
            } else {
                Vector2::new(-50.0, self.base().get_offset().y)
            };

            let vel = cur_pos.lerp(target, 3.0 * delta);
            self.base_mut().set_offset(vel);
        }
    }

    fn ready(&mut self) {
        self.original_offset = self.base().get_offset();
        self.noise.seed = godot::global::randi() as i32;
        // TODO: Learn more about noise generation.
        // Use wrapping math operations to prevent overflow?
        // BUG: This causes subtraction overflow
        // self.noise.frequency = 4.0;
        self.noise.octaves = 2;
        self.noise
            .set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    }
}

#[godot_api]
impl PlayerCamera {
    pub fn add_trauma(&mut self, amount: TraumaLevel) {
        let level = match amount {
            TraumaLevel::Low => 0.25,
            TraumaLevel::Med => 0.5,
            TraumaLevel::High => 0.7,
        };
        self.trauma = (self.trauma + level).clamp(0.0, 1.0);
    }

    fn rust_shake(&mut self) {
        let amount = self.trauma.powf(2.0);
        self.noise_y = self.noise_y.wrapping_add(1);
        let offset_x = self.max_offset.x
            * amount
            * self
                .noise
                .get_noise_2d(self.noise.seed as f32 * 2.0, self.noise_y as f32);

        let offset_y = self.max_offset.y
            * amount
            * self
                .noise
                .get_noise_2d(self.noise.seed as f32 * 3.0, self.noise_y as f32);

        let o = self.original_offset;
        self.base_mut()
            .set_offset(o + Vector2::new(offset_x, offset_y));
    }

    pub fn set_right(&mut self, value: Option<bool>) {
        self.set_right = value;

        if let Some(b) = self.set_right {
            if b {
                self.max_offset = Vector2::new(55.0, -55.0);
            } else {
                self.max_offset = Vector2::new(-55.0, -55.0);
            }
        }
    }

    pub fn reset_x_offset(&mut self) {
        let y = self.base().get_offset().y;
        self.base_mut().set_offset(Vector2::new(0.0, y));
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
    #[export_group(name = "CameraProps")]
    y_offset: f32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = -10000000)]
    left: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = -10000000)]
    top: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = 10000000)]
    right: i32,

    #[export]
    #[export_subgroup(name = "Limits")]
    #[init(val = 10000000)]
    bottom: i32,

    #[export]
    #[export_subgroup(name = "Drag")]
    manual_drag_effect: bool,

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

    #[export]
    #[export_subgroup(name = "Position Smoothing")]
    position_smoothing: bool,

    #[export]
    #[export_subgroup(name = "Position Smoothing")]
    position_smoothing_speed: f32,

    prev_horizontal_enabled: Option<bool>,
    prev_vertical_enabled: Option<bool>,
    prev_zoom: Option<Vector2>,
    prev_y_offset: Option<f32>,
    prev_drag_margin: Vec<Option<(Side, f32)>>,
    prev_limits: Vec<Option<(Side, i32)>>,
    prev_smoothing_enabled: Option<bool>,
    prev_smoothing_speed: Option<f32>,
    prev_manual_drag_enabled: Option<bool>,

    base: Base<Area2D>,
}

#[godot_api]
impl CameraData {
    fn apply_camera_properties(&mut self) {
        if let Some(camera) = self.player_camera.as_mut() {
            let offset = camera.get_offset();
            // Obtain current camera properties.
            self.prev_horizontal_enabled = Some(camera.is_drag_horizontal_enabled());
            self.prev_vertical_enabled = Some(camera.is_drag_vertical_enabled());
            self.prev_zoom = Some(camera.get_zoom());
            self.prev_y_offset = Some(offset.y);
            self.prev_manual_drag_enabled = Some(camera.bind().enable_manual_drag);

            self.prev_smoothing_enabled = Some(camera.is_position_smoothing_enabled());
            self.prev_smoothing_speed = Some(camera.get_position_smoothing_speed());

            for s in Side::values() {
                self.prev_drag_margin
                    .push(Some((*s, camera.get_drag_margin(*s))));
                self.prev_limits.push(Some((*s, camera.get_limit(*s))));
            }

            // Apply new camera properties.
            camera.set_zoom(self.zoom);
            camera.set_limit(Side::LEFT, self.left);
            camera.set_limit(Side::TOP, self.top);
            camera.set_limit(Side::RIGHT, self.right);
            camera.set_limit(Side::BOTTOM, self.bottom);
            camera.set_offset(Vector2::new(offset.x, self.y_offset));
            camera.set_drag_horizontal_enabled(self.horiztonal_enabled);
            camera.set_drag_vertical_enabled(self.vertical_enabled);
            camera.bind_mut().enable_manual_drag = self.manual_drag_effect;
            camera.set_position_smoothing_enabled(self.position_smoothing);
            if !self.manual_drag_effect {
                camera.bind_mut().reset_x_offset();
            }

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
            camera.set_position_smoothing_enabled(self.prev_smoothing_enabled.unwrap());
            camera.set_position_smoothing_speed(self.prev_smoothing_speed.unwrap());
            camera.bind_mut().enable_manual_drag = self.prev_manual_drag_enabled.unwrap();
            camera.set_offset(Vector2::new(0.0, self.prev_y_offset.unwrap()));

            for (side, val) in self.prev_limits.iter().flatten() {
                camera.set_limit(*side, *val);
            }
            for (side, val) in self.prev_drag_margin.iter().flatten() {
                camera.set_drag_margin(*side, *val);
            }
        }
    }

    fn on_player_entered(&mut self, _area: Gd<Area2D>) {
        self.apply_camera_properties();
    }

    fn on_player_exit(&mut self, _area: Gd<Area2D>) {
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

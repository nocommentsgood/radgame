use godot::builtin::Vector2;

struct Speeds {
    patrol: f32,
    aggro: f32,
}

struct Movement {
    speeds: Speeds,
    velocity: Vector2,
}
